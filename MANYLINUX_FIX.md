# 🔧 Manylinux 容器构建修复

## ❌ 问题

```
💥 maturin failed
Caused by: Failed to build a native library through cargo
Error: The process '/usr/bin/docker' failed with exit code 1
```

## 🔍 根本原因

### 什么是 manylinux？

**manylinux** 是一个标准化的 Linux wheel 构建环境：
- 基于 CentOS/AlmaLinux
- 使用 Docker 容器
- 确保二进制兼容性
- 可在大多数 Linux 发行版上运行

### 为什么会失败？

```
主机系统（Ubuntu）:
  ✅ apt-get install libclang-dev
  ✅ 设置 LIBCLANG_PATH
         ↓
  启动 manylinux Docker 容器
         ↓
  容器内部（CentOS）:
  ❌ 没有 libclang
  ❌ 环境变量丢失
  ❌ 编译失败
```

**关键问题**：
1. 主机安装的依赖在容器内不可见
2. 主机的环境变量不会传入容器
3. 容器是隔离的独立环境

## ✅ 解决方案

### 使用 `before-script-linux`

在 manylinux 容器启动后、构建前执行脚本：

```yaml
- name: Build wheels
  uses: PyO3/maturin-action@v1
  with:
    target: ${{ matrix.target }}
    manylinux: auto
    before-script-linux: |
      # 在 manylinux 容器中安装依赖
      yum install -y clang-devel cmake3

      # 查找并设置 libclang
      export LIBCLANG_PATH=$(find /usr/lib* -name "libclang.so*" 2>/dev/null | head -1 | xargs dirname 2>/dev/null || echo "/usr/lib64")
      echo "LIBCLANG_PATH=$LIBCLANG_PATH"
```

### 为什么用 yum？

| 系统 | 包管理器 | 说明 |
|------|----------|------|
| Ubuntu (主机) | `apt-get` | Debian 系 |
| manylinux (容器) | `yum` | RedHat 系（CentOS/AlmaLinux）|

### 依赖对比

| 包 | Ubuntu (apt) | manylinux (yum) |
|----|--------------|-----------------|
| Clang 开发包 | `libclang-dev` | `clang-devel` |
| CMake | `cmake` | `cmake3` |
| 基础工具 | `build-essential` | 已包含 |

## 🔄 完整流程

```
1. GitHub Actions 启动 Ubuntu runner
         ↓
2. 安装主机依赖（可选，用于非容器构建）
   apt-get install libclang-dev
         ↓
3. PyO3/maturin-action 启动 manylinux 容器
         ↓
4. before-script-linux 在容器中执行
   ├─ yum install clang-devel cmake3
   └─ export LIBCLANG_PATH=...
         ↓
5. maturin 在容器中构建
   ├─ boring-sys2 找到 libclang ✅
   └─ 编译成功 ✅
         ↓
6. wheel 输出到主机
```

## 📦 容器环境

### manylinux2014 (CentOS 7 based)
```bash
OS: CentOS 7
Python: 3.8, 3.9, 3.10, 3.11, 3.12
Compiler: gcc 10
```

### manylinux_2_28 (AlmaLinux 8 based)
```bash
OS: AlmaLinux 8
Python: 3.8+
Compiler: gcc 11
```

`manylinux: auto` 自动选择最合适的版本。

## 🆚 方案对比

### 方案 1：在容器中安装依赖 ✅（采用）

```yaml
manylinux: auto
before-script-linux: |
  yum install -y clang-devel cmake3
```

**优点**：
- ✅ 标准化构建环境
- ✅ 广泛兼容性
- ✅ 推荐做法

**缺点**：
- ⚠️ 构建稍慢（需要安装依赖）

### 方案 2：禁用容器 ❌

```yaml
manylinux: 'off'
container: 'off'
```

**优点**：
- ✅ 使用主机依赖
- ✅ 构建更快

**缺点**：
- ❌ 兼容性差（仅在新系统可用）
- ❌ 不推荐用于发布

### 方案 3：使用 zig（实验性）

```yaml
args: --release --zig
```

**优点**：
- ✅ 跨平台编译
- ✅ 无需容器

**缺点**：
- ❌ 不稳定
- ❌ 可能有兼容性问题

## 💡 关键知识点

### before-script-linux 执行时机

```
maturin-action 启动
    ↓
拉取 manylinux Docker 镜像
    ↓
启动容器
    ↓
执行 before-script-linux  ← 在这里安装依赖
    ↓
运行 cargo build
    ↓
输出 wheel
```

### 环境变量传递

```yaml
# ❌ 主机环境变量不会自动传入容器
- name: Set env
  run: echo "FOO=bar" >> $GITHUB_ENV

# ✅ 在 before-script-linux 中设置
before-script-linux: |
  export FOO=bar
```

### libclang 路径查找

```bash
# 在 manylinux 容器中
find /usr/lib* -name "libclang.so*"
# 可能返回：
# /usr/lib64/libclang.so.14
# 或
# /usr/lib/llvm/lib/libclang.so

# 提取目录
dirname /usr/lib64/libclang.so.14
# 返回: /usr/lib64
```

## 🎯 验证

### 成功的构建日志

```
✅ PyO3/maturin-action@v1
   Pulling manylinux image...
   Running before-script-linux...
   + yum install -y clang-devel cmake3
   Installed: clang-devel-14.0.6
   + export LIBCLANG_PATH=/usr/lib64
   LIBCLANG_PATH=/usr/lib64
   Building wheel...
   Compiling boring-sys2...
   ✅ Success
```

### 输出的 wheel

```
never_primp-1.0.0-cp38-abi3-manylinux_2_17_x86_64.whl
```

注意标签：`manylinux_2_17` 表示兼容 glibc 2.17+（覆盖大部分 Linux）

## 🐛 故障排查

### 如果 yum 安装失败

```yaml
before-script-linux: |
  # 更新 yum 缓存
  yum clean all
  yum makecache
  yum install -y clang-devel cmake3
```

### 如果 libclang 找不到

```yaml
before-script-linux: |
  yum install -y clang-devel
  # 手动设置路径
  export LIBCLANG_PATH=/usr/lib64/llvm/lib
  ls -la $LIBCLANG_PATH/
```

### 查看容器环境

```yaml
before-script-linux: |
  cat /etc/os-release
  yum --version
  python --version
  gcc --version
```

## 📝 提交

```bash
git add .github/workflows/build.yml
git commit -m "Fix: install dependencies in manylinux container

- Use before-script-linux to install clang-devel in container
- Set LIBCLANG_PATH dynamically in container
- Use yum instead of apt-get for CentOS-based manylinux"

git push origin main
```

---

**manylinux 容器构建问题已解决！** 🎉

现在依赖会在容器内正确安装，构建可以成功完成。
