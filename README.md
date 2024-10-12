# vss-rs
A rust library for reading and writing voxel data and Sparse Voxel Octrees (SVO) ported from [the C++ library vss](https://github.com/cooukiez/vss). It also supports converting voxel data to svo but only cpu-sided.
## Bvox
### Header pattern
```c
u8 version @ 0x00;
u32 chunk_res @ 0x04;child_mask 
u32 chunk_size @ 0x08;
bool run_length_encoded @ 0x0C;
bool morton_encoded @ 0x0D;

u8 data[<data_length>] @ 0x10;
```
### Palette
Coming soon.
### Data Format
Each Voxel is an `u8`, which is a color index into the palette. Currently only `0` or `1` which indicates the voxel is either empty or filled.

## Bsvo
### Header pattern
```c
u8 version @ 0x00;
u8 max_depth @ 0x04;
u32 root_res @ 0x08;
bool run_length_encoded @ 0x0C;
```
### Palette
Coming soon.
### SvoNode Format
```c
first_child_index @ 0x00;
child_mask @ 0x03;
```
first_child_index: bits 0 -> 23\
child_mask: bits 24 -> 31

## Todo
- [ ] octree creation on gpu?
- [ ] palette support
- [ ] more compression algorithms
- [ ] other file formats
- [ ] voxelization with conservative rasterization 