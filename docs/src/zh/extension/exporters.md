# 导出器

exporter 把校验后的配置数据写成运行时数据包。

Exporter 和 code generator 是分离的，因为同一份导出数据可以被多种语言消费。

## 什么时候添加 Exporter

当需要这些能力时添加 exporter：

- 新的运行时 wire format；
- 平台特定 asset package；
- 不同压缩或 section layout；
- 面向工具的检查格式。

不要为了支持一种新编程语言而添加 exporter。那应该添加 code generator。

## Expected Boundary

exporter 应该消费：

- 归一化 schema IR；
- 校验后的 config data；
- exporter options；
- output target。

它不应该依赖某个具体语言生成器。
