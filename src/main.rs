#[macro_use]
extern crate clap;

#[macro_use]
extern crate itertools;

use xmltree::{Element, XMLNode};

fn parse_length(len: &str) -> f64 {
    let n = len.len();
    if len.ends_with("cm") {
        len[..n - 2].parse::<f64>().unwrap() * 10.0
    } else if len.ends_with("mm") {
        len[..n - 2].parse::<f64>().unwrap()
    } else {
        panic!("unhandled length: {}", len);
    }
}

fn main() {
    let matches = clap_app!(svg_panelize =>
      (version: "0.1")
      (author: "Josh Gao <josh@jmgao.dev>")
      (@arg X: -x +takes_value "Number of times to clone in the X axis")
      (@arg X_OFFSET: --("x-offset") +takes_value "Offset to shift each column by")
      (@arg Y: -y +takes_value "Number of times to clone in the X axis")
      (@arg Y_OFFSET: --("y-offset") +takes_value "Offset to shift each row by")
      (@arg INPUT: +required "Input path")
      (@arg OUTPUT: -o +required +takes_value "Output path")
    )
    .get_matches();

    let input_file =
        std::fs::read_to_string(matches.value_of("INPUT").unwrap()).expect("failed to open file");
    let root = Element::parse(input_file.as_bytes()).expect("failed to parse file");
    let width = parse_length(
        root.attributes
            .get("width")
            .expect("failed to find width on svg node"),
    );
    let height = parse_length(
        root.attributes
            .get("height")
            .expect("failed to find height on svg node"),
    );
    let view_box = root
        .attributes
        .get("viewBox")
        .expect("failed to find viewBox on svg node");
    let view_box_parts: Vec<i32> = view_box
        .trim()
        .split(" ")
        .map(|x| x.parse().unwrap())
        .collect();
    let view_box_minx = view_box_parts[0];
    let view_box_miny = view_box_parts[1];
    let view_box_width = view_box_parts[2] as f64;
    let view_box_height = view_box_parts[3] as f64;
    let children: Vec<XMLNode> = root
        .children
        .iter()
        .filter_map(|x| match x {
            XMLNode::Element(elem) => {
                if elem.name == "title" || elem.name == "desc" {
                    None
                } else {
                    Some(XMLNode::Element(elem.clone()))
                }
            }

            _ => None,
        })
        .collect();

    let x: i32 = matches
        .value_of("X")
        .map(|x| x.parse().unwrap())
        .unwrap_or(1);
    let x_offset: f64 = matches
        .value_of("X_OFFSET")
        .and_then(|x| x.parse().ok())
        .unwrap_or(0.0);
    let y: i32 = matches
        .value_of("Y")
        .map(|x| x.parse().unwrap())
        .unwrap_or(1);
    let y_offset: f64 = matches
        .value_of("Y_OFFSET")
        .and_then(|x| x.parse().ok())
        .unwrap_or(0.0);

    let x_offset = x_offset * view_box_width / width;
    let y_offset = y_offset * view_box_height / height;

    let mut new_children = Vec::new();
    for (i, j) in iproduct!(0..x, 0..y) {
        let x_shift = i as f64 * x_offset;
        let y_shift = j as f64 * y_offset;
        let mut group = Element::new("g");
        group.attributes.insert(
            "transform".to_string(),
            format!("translate({},{})", x_shift, y_shift),
        );
        group.children = children.clone();
        new_children.push(XMLNode::Element(group));
    }

    let mut new_root = root.clone();
    new_root.children = new_children;
    *new_root.attributes.get_mut("width").unwrap() = format!("{}mm", width * x as f64);
    *new_root.attributes.get_mut("height").unwrap() = format!("{}mm", height * y as f64);
    *new_root.attributes.get_mut("viewBox").unwrap() = format!(
        "{} {} {} {}",
        view_box_minx,
        view_box_miny,
        view_box_width * x as f64,
        view_box_height * y as f64
    );
    let output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(matches.value_of("OUTPUT").unwrap())
        .expect("failed to open output file");
    new_root
        .write(&output_file)
        .expect("failed to write to file");
}
