# 🔧 LIBCLANG_PATH 问题修复

## ❌ 持续的错误

```
Unable to find libclang: "couldn't find any valid shared libraries
matching: ['libclang.so', 'libclang-*.so', ...]"
```

## 🔍 问题分析

### 为什么还是找不到？

虽然安装了 `libclang-dev`，但 bindgen 不知道去哪里找：

```
安装的包：
/usr/lib/llvm-14/lib/libclang.so.1
/usr/lib/llvm-15/lib/libclang.so.1
/usr/lib/x86_64-linux-gnu/libclang-14.so.1

bindgen 查找路径：
/usr/lib/libclang.so  ← 找不到！
```

**问题**：libclang 在特定版本的子目录中，bindgen 默认查找路径找不到。

## ✅ 解决方案

### 动态查找 libclang

不硬编码路径（因为 LLVM 版本可能变化），而是动态查找：

```yaml
- name: Find and set libclang path
  run: |
    # 查找第一个 libclang.so
    LIBCLANG_PATH=$(find /usr/lib -name "libclang.so*" | head -1 | xargs dirname)

    # 设置环境变量
    echo "LIBCLANG_PATH=$LIBCLANG_PATH" >> $GITHUB_ENV

    # 调试输出
    echo "Found libclang at: $LIBCLANG_PATH"
    ls -la $LIBCLANG_PATH/libclang.so*
```

### 工作原理

1. **查找**：`find /usr/lib -name "libclang.so*"`
   - 在 `/usr/lib` 下递归查找
   - 找到所有 `libclang.so*` 文件

2. **取第一个**：`head -1`
   - 可能有多个版本，取第一个

3. **获取目录**：`xargs dirname`
   - 提取文件所在目录
   - 例如：`/usr/lib/llvm-14/lib`

4. **设置环境变量**：
   ```bash
   LIBCLANG_PATH=/usr/lib/llvm-14/lib
   ```

5. **bindgen 使用**：
   - 自动读取 `LIBCLANG_PATH` 环境变量
   - 在该路径查找 `libclang.so`

## 🔄 完整流程

```yaml
build-linux:
  steps:
    # 1. 安装依赖
    - name: Install dependencies
      run: |
        sudo apt-get install -y libclang-dev clang cmake

    # 2. 动态查找并设置 libclang 路径
    - name: Find and set libclang path
      run: |
        LIBCLANG_PATH=$(find /usr/lib -name "libclang.so*" | head -1 | xargs dirname)
        echo "LIBCLANG_PATH=$LIBCLANG_PATH" >> $GITHUB_ENV

    # 3. 构建（bindgen 现在可以找到 libclang）
    - name: Build wheels
      uses: PyO3/maturin-action@v1
```

## 🆚 其他方案对比

### 方案 1：硬编码路径 ❌

```yaml
echo "LIBCLANG_PATH=/usr/lib/llvm-14/lib" >> $GITHUB_ENV
```

**缺点**：
- Ubuntu 更新后 LLVM 版本可能从 14 变成 15/16
- 不同 runner 可能有不同版本
- 维护困难

### 方案 2：动态查找 ✅（采用）

```yaml
LIBCLANG_PATH=$(find /usr/lib -name "libclang.so*" | head -1 | xargs dirname)
```

**优点**：
- ✅ 自动适配任何 LLVM 版本
- ✅ 在任何 Ubuntu 版本上工作
- ✅ 无需维护

### 方案 3：使用特定 LLVM 版本

```yaml
sudo apt-get install -y llvm-14-dev libclang-14-dev
echo "LIBCLANG_PATH=/usr/lib/llvm-14/lib" >> $GITHUB_ENV
```

**优点**：明确版本
**缺点**：需要维护版本号

## 📊 测试验证

### 预期日志输出

```
Found libclang at: /usr/lib/llvm-14/lib
-rwxr-xr-x 1 root root 12345678 Jan 1 12:00 /usr/lib/llvm-14/lib/libclang.so
-rwxr-xr-x 1 root root 12345678 Jan 1 12:00 /usr/lib/llvm-14/lib/libclang.so.1
```

### 构建应该成功

1. ✅ 找到 libclang
2. ✅ 设置 LIBCLANG_PATH
3. ✅ bindgen 成功生成绑定
4. ✅ boring-sys2 编译成功
5. ✅ wheel 构建完成

## 💡 为什么要这样做？

### bindgen 的查找逻辑

```rust
// bindgen 内部查找顺序
1. 检查 LIBCLANG_PATH 环境变量 ← 我们设置这个
2. 检查系统默认路径
   - /usr/lib/libclang.so
   - /usr/local/lib/libclang.so
3. 如果都找不到 → 报错
```

我们通过设置 `LIBCLANG_PATH` 直接告诉 bindgen 在哪里找，避免它在默认路径找不到。

## 🐛 如果还是失败？

### 调试步骤

在 workflow 中添加调试信息：

```yaml
- name: Debug libclang
  run: |
    echo "=== Searching for libclang ==="
    find /usr -name "libclang.so*" 2>/dev/null || true

    echo "=== Installed clang packages ==="
    dpkg -l | grep clang

    echo "=== LIBCLANG_PATH ==="
    echo $LIBCLANG_PATH

    echo "=== Files in LIBCLANG_PATH ==="
    ls -la $LIBCLANG_PATH/ || true
```

### 备用方案

如果动态查找失败，尝试安装特定版本：

```yaml
- name: Install specific LLVM version
  run: |
    sudo apt-get install -y llvm-15-dev libclang-15-dev
    echo "LIBCLANG_PATH=/usr/lib/llvm-15/lib" >> $GITHUB_ENV
```

## 📝 提交更改

```bash
git add .github/workflows/build.yml
git commit -m "Fix: dynamically find and set LIBCLANG_PATH for bindgen"
git push origin main
```

---

**这次应该彻底解决了！** 🎯

动态查找确保在任何环境下都能找到 libclang，不依赖特定版本号。
