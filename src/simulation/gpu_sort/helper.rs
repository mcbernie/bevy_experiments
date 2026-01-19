pub fn calc_num_groups(item_count: u32, group_size: u32) -> u32 {
    ((item_count + (group_size * 2 - 1)) / (group_size * 2)).max(1)
}