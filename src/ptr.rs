use crate::{
    green::GreenNode,
    red::{RedNode, RedNodeData},
};

struct Ptr {
    seq_of_steps: Vec<usize>,
    range: (usize, usize),
}

impl Ptr {
    fn addr_of(node: &RedNode) -> Ptr {
        todo!()
    }

    fn deref(self, root_node: &RedNode) -> RedNode {
        todo!()
    }
}

// #[test]
// fn smoke() {
//     let green: GreenNode = None.unwrap();
//     let chan = None.unwrap();
//     {
//         let root = RedNodeData::new_root(green);
//         let red = root.children().filter_map(());
//         let ptr = Ptr::addr_of(&red);
//         chan.send(ptr);
//     }
//     {
//         let root = RedNodeData::new_root(green);

//         let ptr: Ptr = chan.recv(ptr);
//         let red: RedNode = ptr.deref(&root);
//     }
// }
