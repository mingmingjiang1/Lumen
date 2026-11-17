// 性能示例：按 js-framework-benchmark 的操作集裁剪出来的最小版本。
// - 初始数据 = 「create 200 rows」：一次建 200 行的视图（行数上限由运行期数据容量 256 决定）；
// - 追加 = append row（push 一行，整表重建）；
// - 删除 = remove row（removeAt，整表重建）；
// - 清空 = clear rows（整表赋值，整表重建）；
// - 选中 = update row（行内 `item.selected = true`，只重建该表）。
// 这个页面本身不做计时：时间用外部工具量（Instruments / gfxinfo / Chrome perf），
// 可参考 README「性能示例与测量」一节的命令。
// 列宽都用百分比 / 固定点值，避免依赖 flexGrow 在行内的分配行为（三端语义还不完全一致）。
function App() {
    const rows = state([
        { id: 1, label: "Item 1", selected: false },
        { id: 2, label: "Item 2", selected: false },
        { id: 3, label: "Item 3", selected: false },
        { id: 4, label: "Item 4", selected: false },
        { id: 5, label: "Item 5", selected: false },
        { id: 6, label: "Item 6", selected: false },
        { id: 7, label: "Item 7", selected: false },
        { id: 8, label: "Item 8", selected: false },
        { id: 9, label: "Item 9", selected: false },
        { id: 10, label: "Item 10", selected: false },
        { id: 11, label: "Item 11", selected: false },
        { id: 12, label: "Item 12", selected: false },
        { id: 13, label: "Item 13", selected: false },
        { id: 14, label: "Item 14", selected: false },
        { id: 15, label: "Item 15", selected: false },
        { id: 16, label: "Item 16", selected: false },
        { id: 17, label: "Item 17", selected: false },
        { id: 18, label: "Item 18", selected: false },
        { id: 19, label: "Item 19", selected: false },
        { id: 20, label: "Item 20", selected: false },
        { id: 21, label: "Item 21", selected: false },
        { id: 22, label: "Item 22", selected: false },
        { id: 23, label: "Item 23", selected: false },
        { id: 24, label: "Item 24", selected: false },
        { id: 25, label: "Item 25", selected: false },
        { id: 26, label: "Item 26", selected: false },
        { id: 27, label: "Item 27", selected: false },
        { id: 28, label: "Item 28", selected: false },
        { id: 29, label: "Item 29", selected: false },
        { id: 30, label: "Item 30", selected: false },
        { id: 31, label: "Item 31", selected: false },
        { id: 32, label: "Item 32", selected: false },
        { id: 33, label: "Item 33", selected: false },
        { id: 34, label: "Item 34", selected: false },
        { id: 35, label: "Item 35", selected: false },
        { id: 36, label: "Item 36", selected: false },
        { id: 37, label: "Item 37", selected: false },
        { id: 38, label: "Item 38", selected: false },
        { id: 39, label: "Item 39", selected: false },
        { id: 40, label: "Item 40", selected: false },
        { id: 41, label: "Item 41", selected: false },
        { id: 42, label: "Item 42", selected: false },
        { id: 43, label: "Item 43", selected: false },
        { id: 44, label: "Item 44", selected: false },
        { id: 45, label: "Item 45", selected: false },
        { id: 46, label: "Item 46", selected: false },
        { id: 47, label: "Item 47", selected: false },
        { id: 48, label: "Item 48", selected: false },
        { id: 49, label: "Item 49", selected: false },
        { id: 50, label: "Item 50", selected: false },
        { id: 51, label: "Item 51", selected: false },
        { id: 52, label: "Item 52", selected: false },
        { id: 53, label: "Item 53", selected: false },
        { id: 54, label: "Item 54", selected: false },
        { id: 55, label: "Item 55", selected: false },
        { id: 56, label: "Item 56", selected: false },
        { id: 57, label: "Item 57", selected: false },
        { id: 58, label: "Item 58", selected: false },
        { id: 59, label: "Item 59", selected: false },
        { id: 60, label: "Item 60", selected: false },
        { id: 61, label: "Item 61", selected: false },
        { id: 62, label: "Item 62", selected: false },
        { id: 63, label: "Item 63", selected: false },
        { id: 64, label: "Item 64", selected: false },
        { id: 65, label: "Item 65", selected: false },
        { id: 66, label: "Item 66", selected: false },
        { id: 67, label: "Item 67", selected: false },
        { id: 68, label: "Item 68", selected: false },
        { id: 69, label: "Item 69", selected: false },
        { id: 70, label: "Item 70", selected: false },
        { id: 71, label: "Item 71", selected: false },
        { id: 72, label: "Item 72", selected: false },
        { id: 73, label: "Item 73", selected: false },
        { id: 74, label: "Item 74", selected: false },
        { id: 75, label: "Item 75", selected: false },
        { id: 76, label: "Item 76", selected: false },
        { id: 77, label: "Item 77", selected: false },
        { id: 78, label: "Item 78", selected: false },
        { id: 79, label: "Item 79", selected: false },
        { id: 80, label: "Item 80", selected: false },
        { id: 81, label: "Item 81", selected: false },
        { id: 82, label: "Item 82", selected: false },
        { id: 83, label: "Item 83", selected: false },
        { id: 84, label: "Item 84", selected: false },
        { id: 85, label: "Item 85", selected: false },
        { id: 86, label: "Item 86", selected: false },
        { id: 87, label: "Item 87", selected: false },
        { id: 88, label: "Item 88", selected: false },
        { id: 89, label: "Item 89", selected: false },
        { id: 90, label: "Item 90", selected: false },
        { id: 91, label: "Item 91", selected: false },
        { id: 92, label: "Item 92", selected: false },
        { id: 93, label: "Item 93", selected: false },
        { id: 94, label: "Item 94", selected: false },
        { id: 95, label: "Item 95", selected: false },
        { id: 96, label: "Item 96", selected: false },
        { id: 97, label: "Item 97", selected: false },
        { id: 98, label: "Item 98", selected: false },
        { id: 99, label: "Item 99", selected: false },
        { id: 100, label: "Item 100", selected: false },
        { id: 101, label: "Item 101", selected: false },
        { id: 102, label: "Item 102", selected: false },
        { id: 103, label: "Item 103", selected: false },
        { id: 104, label: "Item 104", selected: false },
        { id: 105, label: "Item 105", selected: false },
        { id: 106, label: "Item 106", selected: false },
        { id: 107, label: "Item 107", selected: false },
        { id: 108, label: "Item 108", selected: false },
        { id: 109, label: "Item 109", selected: false },
        { id: 110, label: "Item 110", selected: false },
        { id: 111, label: "Item 111", selected: false },
        { id: 112, label: "Item 112", selected: false },
        { id: 113, label: "Item 113", selected: false },
        { id: 114, label: "Item 114", selected: false },
        { id: 115, label: "Item 115", selected: false },
        { id: 116, label: "Item 116", selected: false },
        { id: 117, label: "Item 117", selected: false },
        { id: 118, label: "Item 118", selected: false },
        { id: 119, label: "Item 119", selected: false },
        { id: 120, label: "Item 120", selected: false },
        { id: 121, label: "Item 121", selected: false },
        { id: 122, label: "Item 122", selected: false },
        { id: 123, label: "Item 123", selected: false },
        { id: 124, label: "Item 124", selected: false },
        { id: 125, label: "Item 125", selected: false },
        { id: 126, label: "Item 126", selected: false },
        { id: 127, label: "Item 127", selected: false },
        { id: 128, label: "Item 128", selected: false },
        { id: 129, label: "Item 129", selected: false },
        { id: 130, label: "Item 130", selected: false },
        { id: 131, label: "Item 131", selected: false },
        { id: 132, label: "Item 132", selected: false },
        { id: 133, label: "Item 133", selected: false },
        { id: 134, label: "Item 134", selected: false },
        { id: 135, label: "Item 135", selected: false },
        { id: 136, label: "Item 136", selected: false },
        { id: 137, label: "Item 137", selected: false },
        { id: 138, label: "Item 138", selected: false },
        { id: 139, label: "Item 139", selected: false },
        { id: 140, label: "Item 140", selected: false },
        { id: 141, label: "Item 141", selected: false },
        { id: 142, label: "Item 142", selected: false },
        { id: 143, label: "Item 143", selected: false },
        { id: 144, label: "Item 144", selected: false },
        { id: 145, label: "Item 145", selected: false },
        { id: 146, label: "Item 146", selected: false },
        { id: 147, label: "Item 147", selected: false },
        { id: 148, label: "Item 148", selected: false },
        { id: 149, label: "Item 149", selected: false },
        { id: 150, label: "Item 150", selected: false },
        { id: 151, label: "Item 151", selected: false },
        { id: 152, label: "Item 152", selected: false },
        { id: 153, label: "Item 153", selected: false },
        { id: 154, label: "Item 154", selected: false },
        { id: 155, label: "Item 155", selected: false },
        { id: 156, label: "Item 156", selected: false },
        { id: 157, label: "Item 157", selected: false },
        { id: 158, label: "Item 158", selected: false },
        { id: 159, label: "Item 159", selected: false },
        { id: 160, label: "Item 160", selected: false },
        { id: 161, label: "Item 161", selected: false },
        { id: 162, label: "Item 162", selected: false },
        { id: 163, label: "Item 163", selected: false },
        { id: 164, label: "Item 164", selected: false },
        { id: 165, label: "Item 165", selected: false },
        { id: 166, label: "Item 166", selected: false },
        { id: 167, label: "Item 167", selected: false },
        { id: 168, label: "Item 168", selected: false },
        { id: 169, label: "Item 169", selected: false },
        { id: 170, label: "Item 170", selected: false },
        { id: 171, label: "Item 171", selected: false },
        { id: 172, label: "Item 172", selected: false },
        { id: 173, label: "Item 173", selected: false },
        { id: 174, label: "Item 174", selected: false },
        { id: 175, label: "Item 175", selected: false },
        { id: 176, label: "Item 176", selected: false },
        { id: 177, label: "Item 177", selected: false },
        { id: 178, label: "Item 178", selected: false },
        { id: 179, label: "Item 179", selected: false },
        { id: 180, label: "Item 180", selected: false },
        { id: 181, label: "Item 181", selected: false },
        { id: 182, label: "Item 182", selected: false },
        { id: 183, label: "Item 183", selected: false },
        { id: 184, label: "Item 184", selected: false },
        { id: 185, label: "Item 185", selected: false },
        { id: 186, label: "Item 186", selected: false },
        { id: 187, label: "Item 187", selected: false },
        { id: 188, label: "Item 188", selected: false },
        { id: 189, label: "Item 189", selected: false },
        { id: 190, label: "Item 190", selected: false },
        { id: 191, label: "Item 191", selected: false },
        { id: 192, label: "Item 192", selected: false },
        { id: 193, label: "Item 193", selected: false },
        { id: 194, label: "Item 194", selected: false },
        { id: 195, label: "Item 195", selected: false },
        { id: 196, label: "Item 196", selected: false },
        { id: 197, label: "Item 197", selected: false },
        { id: 198, label: "Item 198", selected: false },
        { id: 199, label: "Item 199", selected: false },
        { id: 200, label: "Item 200", selected: false }
    ]);
    const draft = state("Item");
    const nextId = state(201);
    // onScroll={(y) => offset = clamp(y, 0, 100)}：滚动位置写进数值单元，滚动驱动的效果就能用普通 TSX 写
    const offset = state(0);
    return <View style={{ padding: 16, backgroundColor: "rgb(244, 245, 247)", width: "100%" }}>
        <Text style={{ fontSize: 20, color: "rgb(28, 39, 61)" }}>Bench（200 行）</Text>
        <Text style={{ marginTop: 4, fontSize: 12, color: "rgb(138, 148, 166)" }}>{rows.length}</Text>
        <Text style={{ marginTop: 2, fontSize: 12, color: "rgb(47, 107, 255)" }}>{offset}</Text>
        <Text style={{ marginTop: 2, fontSize: 12, color: "rgb(35, 155, 110)", opacity: offset }}>offset 夹在 0..100：滚到 100px 后这里完全显现</Text>
        <View style={{ flexDirection: "row", marginTop: 10, width: "100%", justifyContent: "space-between" }}>
            <Button
                onClick={() => {
                    rows.push({ id: nextId, label: draft, selected: false });
                    nextId++;
                }}
                style={{ height: 36, width: "50%", backgroundColor: "rgb(47, 107, 255)", borderRadius: 8 }}
            >
                <Text style={{ color: "white", fontSize: 14 }}>追加</Text>
            </Button>
            <Button
                onClick={() => rows = []}
                style={{ height: 36, width: "50%", backgroundColor: "white", borderRadius: 8, color: "rgb(138, 148, 166)" }}
            >
                <Text style={{ fontSize: 14 }}>清空</Text>
            </Button>
        </View>
        <ScrollView style={{ height: 460, marginTop: 12, width: "100%" }} onScroll={(y) => offset = clamp(y, 0, 100)}>
            {rows.map((item, index) => (
                <View style={{
                    flexDirection: "row",
                    alignItems: "center",
                    height: 36,
                    width: "100%",
                    paddingHorizontal: 10,
                    marginBottom: 4,
                    backgroundColor: "white",
                    borderRadius: 6
                }}>
                    <Text style={{ width: 40, fontSize: 11, color: "rgb(138, 148, 166)" }}>{item.id}</Text>
                    <Text style={{ width: 150, fontSize: 13, color: "rgb(28, 39, 61)" }}>{item.label}</Text>
                    {item.selected
                        ? <Text style={{ width: 44, fontSize: 11, color: "rgb(35, 155, 110)" }}>已选</Text>
                        : <Text style={{ width: 44, fontSize: 11, color: "rgb(170, 178, 190)" }}>未选</Text>}
                    <Button
                        onClick={() => item.selected = true}
                        style={{ height: 24, width: 44, backgroundColor: "rgb(242, 243, 245)", borderRadius: 6, color: "rgb(47, 107, 255)" }}
                    >
                        <Text style={{ fontSize: 11 }}>选中</Text>
                    </Button>
                    <Button
                        onClick={() => rows.removeAt(index)}
                        style={{ height: 24, width: 44, marginLeft: 6, backgroundColor: "rgb(242, 243, 245)", borderRadius: 6, color: "rgb(138, 148, 166)" }}
                    >
                        <Text style={{ fontSize: 11 }}>删除</Text>
                    </Button>
                </View>
            ))}
        </ScrollView>
    </View>;
}
