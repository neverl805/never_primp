# 🔧 Python 包导入问题修复

## ❌ 错误信息

```python
ModuleNotFoundError: No module named 'never_primp.never_primp'
```

## 🔍 问题分析

### 预期的包结构

```
wheel 包内应该是:
never_primp/
├── __init__.py
├── never_primp.pyd (或 .so)
└── never_primp.pyi
```

### 实际发生了什么

```python
# never_primp/__init__.py
from .never_primp import RClient  # ← 找不到 never_primp.pyd
```

**原因**：wheel 打包时 `never_primp/` 文件夹没有被正确包含。

## 🔧 Maturin 配置说明

### module-name 的作用

```toml
[tool.maturin]
module-name = "never_primp.never_primp"
```

这告诉 maturin：
- 包名：`never_primp`
- 编译的模块名：`never_primp.never_primp`
- 最终位置：`never_primp/never_primp.pyd`

### 目录结构要求

```
项目根目录/
├── Cargo.toml
├── pyproject.toml
├── src/
│   └── lib.rs
└── never_primp/        ← 必须存在！
    ├── __init__.py
    ├── *.pyi
    └── py.typed
```

Maturin 会自动：
1. 编译 Rust 代码 → `never_primp.pyd`
2. 查找 `never_primp/` 文件夹
3. 将 `.pyd` 和 `__init__.py` 打包在一起

## ✅ 解决方案

### 方案 1：确保 never_primp/ 文件夹存在（推荐）

```bash
# 检查文件夹
ls never_primp/

# 应该包含
never_primp/
├── __init__.py  ✅
├── never_primp.pyi  ✅ (可选)
└── py.typed  ✅ (可选)
```

### 方案 2：简化 pyproject.toml

```toml
[tool.maturin]
module-name = "never_primp.never_primp"
features = ["pyo3/extension-module"]

# 移除 python-source（让 maturin 自动发现）
```

### 方案 3：验证构建

```bash
# 本地构建测试
maturin develop

# 测试导入
python -c "from never_primp import RClient; print('✅ OK')"

# 检查打包内容
maturin build --release
unzip -l target/wheels/*.whl | grep never_primp
```

## 📦 正确的 Wheel 内容

```bash
$ unzip -l never_primp-1.0.0-cp38-abi3-win_amd64.whl

Archive:  never_primp-1.0.0-cp38-abi3-win_amd64.whl
  Length      Date    Time    Name
---------  ---------- -----   ----
    12810  2025-01-01 12:00   never_primp/__init__.py
  6435328  2025-01-01 12:00   never_primp/never_primp.pyd  ← 编译的二进制
     5731  2025-01-01 12:00   never_primp/never_primp.pyi
        0  2025-01-01 12:00   never_primp/py.typed
     1234  2025-01-01 12:00   never_primp-1.0.0.dist-info/METADATA
      ...
```

**关键**：`never_primp/__init__.py` 和 `never_primp/never_primp.pyd` 必须都在！

## 🐛 如果还是失败

### 检查 1：never_primp/ 文件夹是否被 git 跟踪

```bash
git status never_primp/
git add never_primp/
```

### 检查 2：.gitignore 是否排除了文件

```bash
# 检查 .gitignore
cat .gitignore | grep never_primp

# 如果有类似规则，移除或更新：
# never_primp/  ← 这会排除整个文件夹！
```

### 检查 3：CI 中文件夹是否存在

在 workflow 中添加调试：

```yaml
- name: Debug package structure
  run: |
    echo "=== Root directory ==="
    ls -la

    echo "=== never_primp directory ==="
    ls -la never_primp/

    echo "=== After build ==="
    maturin build --release
    unzip -l target/wheels/*.whl
```

## 🔄 正确的构建流程

```
1. Cargo 编译 Rust 代码
   src/lib.rs → never_primp.pyd

2. Maturin 查找 Python 文件
   发现 never_primp/__init__.py

3. Maturin 打包
   never_primp/__init__.py  → wheel
   never_primp.pyd          → wheel

4. 安装 wheel
   解压到 site-packages/never_primp/

5. Python 导入
   import never_primp
   → 读取 __init__.py
   → from .never_primp import RClient
   → 加载 never_primp.pyd ✅
```

## 📝 当前配置

### pyproject.toml (修复后)

```toml
[tool.maturin]
module-name = "never_primp.never_primp"
features = ["pyo3/extension-module"]
```

**说明**：
- ✅ 简洁配置
- ✅ 让 maturin 自动发现 `never_primp/` 文件夹
- ✅ 自动包含 `__init__.py` 等文件

### Cargo.toml

```toml
[lib]
name = "never_primp"
crate-type = ["cdylib"]
```

**说明**：
- `name = "never_primp"` → 编译出 `never_primp.pyd`
- maturin 根据 `module-name` 放到正确位置

## ✅ 验证

```bash
# 1. 清理
rm -rf target/wheels

# 2. 构建
maturin build --release

# 3. 检查内容
unzip -l target/wheels/*.whl | grep -E "(__init__|never_primp\.(pyd|so))"

# 应该看到：
# never_primp/__init__.py
# never_primp/never_primp.pyd

# 4. 安装测试
pip install target/wheels/*.whl --force-reinstall

# 5. 测试导入
python -c "from never_primp import RClient; print('✅ Success')"
```

## 📋 提交

```bash
git add pyproject.toml
git add never_primp/__init__.py
git add never_primp/*.pyi
git commit -m "Fix: ensure never_primp Python package is included in wheel"
git push origin main
```

---

**包结构问题已修复！** 🎉

确保 `never_primp/` 文件夹及其内容被正确跟踪和打包。
