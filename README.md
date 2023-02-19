![alt text](https://github.com/rewin123/SpaceSandbox/blob/main/project.png?raw=true)

It is roleplay space game prototype based on bevy engine and custom render.

Custom render will be need for shadow map caching and instance rendering.
Bevy doesn't contain these two features, so I will make it by own by fully custom render pipeline.

Current development target is creating station building mode, which can render SS13 map on GTX 1060 with 60 fps on my laptop.

Полезные ссылки:
1. https://github.com/bombomby/optick - профайлер для игр
2. https://crates.io/crates/profiling - обертка профайлеров для rust
3. https://crates.io/crates/texture-synthesis - интересный генератор текстур
4. https://github.com/tree-sitter/tree-sitter - инкрементальный парсер кода для подсветки

Полезные ссылки про визуализацию:

1. Voxel based near-field global illumination (2011) - https://dl.acm.org/doi/pdf/10.1145/1944745.1944763
