# Excel 表头投影

Excel 模板由归一化 schema 生成。表头是投影，不是独立的格式定义。

## 为什么生成表头

手工维护的 spreadsheet header 很容易和代码漂移：

- 字段在代码里改名了，但 Excel 没改；
- 类型变了，但旧行看起来仍然有效；
- 策划新增一列，但运行时没人读取；
- 校验规则只写在注释里，而没有被执行。

Sora 通过从 schema 生成工作簿结构来避免这些问题。

## 表头包含什么

生成行包含：

- 表元数据：表名、mode、key、scope 和 schema hash；
- 稳定字段名；
- 类型提示；
- scope 提示；
- validation 和 parser rule；
- 给编辑者看的注释。

只有行数据应该被视为作者内容。表头行可以在 schema 变更后重新生成。

## 实际工作流

1. 修改 schema。
2. 重新生成 Excel 模板。
3. 把已有数据行移动或粘贴到更新后的模板中。
4. 运行 `sora build` 或 `sora export` 校验值和引用。
5. 生成导出数据和代码。

这样 Excel 仍然适合编辑，但 schema 始终保持权威。
