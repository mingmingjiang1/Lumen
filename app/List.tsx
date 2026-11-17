// 组件内的 state 按实例分配：两个 Counter 各自计数，互不影响。
function Counter({ label }) {
    const hits = state(0);
    return <View style={{ flexDirection: "row", alignItems: "center", marginTop: 8 }}>
        <Text style={{ fontSize: 13, color: "rgb(138, 148, 166)" }}>{label}</Text>
        <Text style={{ marginLeft: 6, fontSize: 13, color: "rgb(28, 39, 61)" }}>{hits}</Text>
        <Button
            onClick={() => hits++}
            style={{
                marginLeft: 8,
                width: 40,
                height: 28,
                backgroundColor: "white",
                borderRadius: 6,
                color: "rgb(47, 107, 255)"
            }}
        >
            <Text style={{ fontSize: 14 }}>+</Text>
        </Button>
    </View>;
}

// 列表能力演示：数组状态 + .map() 行渲染 + 多语句 handler + 嵌套状态 + 整体赋值。
// - `items` 是对象数组状态，行内通过 `item.title` 读取字段；
// - `{items.length}` 是列表长度单元，push / removeAt / 清空都会同步；
// - `stats.today.added` 是多层对象状态里的一格；
// - 「清空」用整表赋值 `items = []`。
function App() {
    const items = state([
        { id: 1, title: "写周报", done: false },
        { id: 2, title: "评审 PR", done: false }
    ]);
    const stats = state({ today: { added: 0 } });
    const draft = state("");
    const nextId = state(3);

    return <View style={{ padding: 20, backgroundColor: "rgb(244, 245, 247)" }}>
        <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>待办清单</Text>
        <Text style={{ marginTop: 4, fontSize: 13, color: "rgb(138, 148, 166)" }}>{items.length}</Text>
        <Text style={{ marginTop: 2, fontSize: 13, color: "rgb(138, 148, 166)" }}>{stats.today.added}</Text>
        <Text style={{ marginTop: 2, fontSize: 13, color: "rgb(47, 107, 255)" }}>{stats.today.added + items.length}</Text>
        <Counter label="完成数" />
        <Counter label="归档数" />

        <Input
            placeholder="输入待办"
            onInput={(value) => draft = value}
            style={{
                marginTop: 12,
                height: 44,
                backgroundColor: "white",
                borderRadius: 8,
                paddingHorizontal: 12,
                fontSize: 15
            }}
        />

        <Button
            onClick={() => {
                items.push({ id: nextId, title: draft, done: false });
                nextId++;
                stats.today.added++;
                draft = "";
            }}
            style={{
                marginTop: 10,
                height: 44,
                backgroundColor: "rgb(47, 107, 255)",
                borderRadius: 8
            }}
        >
            <Text style={{ color: "white", fontSize: 16 }}>添加</Text>
        </Button>

        <Button
            onClick={() => items = []}
            style={{
                marginTop: 8,
                height: 40,
                backgroundColor: "white",
                borderRadius: 8,
                color: "rgb(138, 148, 166)"
            }}
        >
            <Text style={{ fontSize: 15 }}>清空</Text>
        </Button>

        {/* 视口高度要固定，行数变化才会变成滚动内容 */}
        <ScrollView style={{ height: 380, marginTop: 12 }}>
            {items.map((item, index) => (
                <View style={{
                    flexDirection: "row",
                    justifyContent: "space-between",
                    alignItems: "center",
                    paddingHorizontal: 12,
                    height: 52,
                    backgroundColor: "white",
                    borderRadius: 8,
                    marginBottom: 8
                }}>
                    <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)" }}>{item.title}</Text>
                    {item.done
                        ? <Text style={{ fontSize: 13, color: "rgb(52, 199, 89)" }}>已完成</Text>
                        : <Text style={{ fontSize: 13, color: "rgb(138, 148, 166)" }}>待办</Text>}
                    <Button
                        onClick={() => item.done = true}
                        style={{ height: 28, width: 52, backgroundColor: "rgb(242, 243, 245)", borderRadius: 6, color: "rgb(47, 107, 255)" }}
                    >
                        <Text style={{ fontSize: 12 }}>完成</Text>
                    </Button>
                    <Button
                        onClick={() => items.removeAt(index)}
                        style={{ height: 28, width: 52, backgroundColor: "rgb(242, 243, 245)", borderRadius: 6, color: "rgb(138, 148, 166)" }}
                    >
                        <Text style={{ fontSize: 12 }}>删除</Text>
                    </Button>
                </View>
            ))}
        </ScrollView>
    </View>;
}
