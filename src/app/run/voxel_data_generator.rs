use super::gpu_shared_data::VoxelData;

pub fn generate_tree(layer_count: u8) -> Vec<VoxelData> {
    let mut dst = Vec::new();

    generate_tree_layer_rec(&mut dst, 0, layer_count);

    println!("{:?}", dst[0]);
    println!("{:?}", dst[37]);

    dst
}

fn generate_tree_layer_rec(dst: &mut Vec<VoxelData>, layer_idx: u8, max_layer: u8) -> isize {
    if layer_idx == max_layer {
        return -1;
    }

    dst.push(VoxelData { child_indicies: [0, 0, 0, 0, 0, 0, 0, 0], pallete_idx: 0 });

    let data_index = dst.len() - 1;

    
    dst[data_index].child_indicies[0] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    //dst[data_index].child_indicies[1] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    dst[data_index].child_indicies[2] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    //dst[data_index].child_indicies[3] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;

    //dst[data_index].child_indicies[4] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    dst[data_index].child_indicies[5] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    //dst[data_index].child_indicies[6] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;
    dst[data_index].child_indicies[7] = generate_tree_layer_rec(dst, layer_idx + 1, max_layer).max(0) as u32;


    data_index as isize
}


#[test]
fn test_generation() {
    let tree = generate_tree(3);

    println!("{tree:?}");
}