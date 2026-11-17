// 递归组件示例：TreeRow 渲染一行，并把 `node.children` 交还给自己。
// 编译期只生成一份行模板，子树在运行期按行表的父子关系逐层展开。
function TreeRow({ node }) {
    return <View style={{ flexDirection: "column", spacing: 6 }}>
        <View style={{
            flexDirection: "row",
            alignItems: "center",
            height: 40,
            paddingHorizontal: 12,
            borderRadius: 8,
            backgroundColor: "rgb(244, 245, 247)"
        }}>
            <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)" }}>{node.title}</Text>
        </View>
        <TreeRow node={node.children} />
    </View>;
}

function App() {
    const tree = state([
        {
            title: "前端",
            children: [
                { title: "渲染层" },
                { title: "状态层", children: [{ title: "单元存储" }] }
            ]
        },
        { title: "后端" }
    ]);
    return <View style={{ padding: 20, backgroundColor: "rgb(255, 255, 255)" }}>
        <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>递归组件</Text>
        <Text style={{ marginTop: 4, fontSize: 13, color: "rgb(138, 148, 166)" }}>
            {tree.length}
        </Text>
        <View style={{ marginTop: 16 }}>
            {tree.map((item) => <TreeRow node={item} />)}
        </View>
    </View>;
}
