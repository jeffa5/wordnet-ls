use memmap::Mmap;

pub fn binary_search_file(map: &Mmap, word: &str) -> Option<String> {
    let mut start = 0_usize;
    let mut end = map.len();

    let mut iword = String::new();

    while start < end {
        iword.clear();
        let mut mid = (start + end) / 2;
        // scan forwards to a newline
        while mid < end && map[mid] != b'\n' {
            mid += 1;
        }
        let line_end = mid;
        mid -= 1;
        while mid > start && map[mid] != b'\n' {
            mid -= 1;
        }
        let line_start = mid;

        // mid now points to a newline character so bump it by one to get the start of the line
        mid += 1;

        // now we extract the word from the line
        while mid < end && map[mid] != b' ' {
            iword.push(map[mid] as char);
            mid += 1;
        }
        if mid == end {
            // gone too far
            end = line_start;
            continue;
        }
        if iword.is_empty() {
            // may have been a license line
            start = line_end;
            continue;
        }

        // and check how this word compares to the one we are searching for
        match word.cmp(&iword) {
            std::cmp::Ordering::Less => {
                end = line_start;
            }
            std::cmp::Ordering::Equal => {
                // read the rest of the line into iword
                while map[mid] != b'\n' {
                    iword.push(map[mid] as char);
                    mid += 1;
                }
                // and return the parsed parts
                return Some(iword);
            }
            std::cmp::Ordering::Greater => {
                start = line_end;
            }
        }
    }
    None
}
