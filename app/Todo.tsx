/**
 * Lumen 第二个示例：今日待办。
 *
 * 和 app/App.tsx 用同一套语法，重点演示这个子集里能做的清单交互：
 * - 对象状态展开成独立单元（titles.* / done.* 各自更新）；
 * - 文本状态：输入框直接改写第一项标题；
 * - 布尔状态：逐项勾选 / 撤销，配合条件渲染出「已完成 / 待办」；
 * - 数字状态：番茄计数 +1 / -1 / 归零；
 * - 条件渲染：逐项状态徽标、整块归档卡片；
 * - ScrollView 滚动时顶部栏渐显。
 *
 * 编译：cargo run -- android todo（或 macos todo / 直接 cargo run -- todo）
 */

function App() {
    const titles = state({
        a: "整理 JNI 调用顺序",
        b: "接进 demo 切换脚本",
        c: "补齐双端截图对照",
        d: "写清两个 demo 用法",
    });
    const done = state({ a: false, b: false, c: false, d: false });
    const pomodoro = state(0);
    const archived = state(false);

    return <View style={{
        width: "100%", height: "100%",
        backgroundColor: "rgb(247, 248, 251)",
        flexDirection: "column",
    }}>
        <View style={{
            id: "topBar",
            backgroundColor: "rgb(28, 39, 61)",
            width: "100%", height: 44, padding: 12,
            flexDirection: "row", alignItems: "center", justifyContent: "space-between",
        }}>
            <Text style={{ fontSize: 20, color: "white" }}>今日待办</Text>
            <Text style={{ fontSize: 13, color: "rgb(190, 205, 230)" }}>4 TASKS</Text>
        </View>

        <ScrollView style={{
            fadeTarget: "topBar", fadeStart: 0, fadeDistance: 120,
            marginHorizontal: 16, marginTop: 10, marginBottom: 72,
            flexGrow: 1, padding: 16, spacing: 14,
            backgroundColor: "white", borderRadius: 18,
        }}>
            {/* 进度卡：每一项的完成状态各自驱动一个指示点 */}
            <View style={{
                width: "100%", height: 96, padding: 16, borderRadius: 14,
                backgroundColor: "rgb(69, 104, 220)",
                flexDirection: "row", alignItems: "center", justifyContent: "space-between",
            }}>
                <View style={{ flexDirection: "column", spacing: 6 }}>
                    <Text style={{ fontSize: 13, color: "rgb(214, 225, 255)" }}>TODAY'S PROGRESS</Text>
                    <Text style={{ fontSize: 20, color: "white" }}>逐项勾掉，进度见底</Text>
                </View>
            </View>

            {/* 输入框直接改写第一项标题：字符串状态 + onInput */}
            <Input
                placeholder="改写第一项标题"
                onInput={(value) => titles.a = value}
                style={{
                    width: "100%", height: 48, padding: 12, borderRadius: 12,
                    backgroundColor: "rgb(242, 245, 250)",
                    fontSize: 15, color: "rgb(28, 39, 61)",
                }}
            />

            {/* 任务 1 */}
            <View style={{
                width: "100%", height: 96, padding: 14, borderRadius: 14, spacing: 10,
                backgroundColor: "rgb(249, 250, 253)",
                borderWidth: 1, borderColor: "rgb(232, 236, 244)",
                flexDirection: "column", justifyContent: "space-between",
            }}>
                <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", spacing: 8 }}>
                    <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)", flexGrow: 1 }}>{titles.a}</Text>
                    {done.a ? <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(232, 246, 239)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(35, 155, 110)", textAlign: "center" }}>已完成</Text>
                    </View> : <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(238, 241, 247)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(120, 131, 155)", textAlign: "center" }}>待办</Text>
                    </View>}
                </View>
                <View style={{ flexDirection: "row", spacing: 8 }}>
                    <Button
                        title="完成"
                        onClick={() => done.a = true}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", fontSize: 13 }}
                    />
                    <Button
                        title="撤销"
                        onClick={() => done.a = false}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(232, 236, 244)", color: "rgb(28, 39, 61)", fontSize: 13 }}
                    />
                </View>
            </View>

            {/* 任务 2 */}
            <View style={{
                width: "100%", height: 96, padding: 14, borderRadius: 14, spacing: 10,
                backgroundColor: "rgb(249, 250, 253)",
                borderWidth: 1, borderColor: "rgb(232, 236, 244)",
                flexDirection: "column", justifyContent: "space-between",
            }}>
                <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", spacing: 8 }}>
                    <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)", flexGrow: 1 }}>{titles.b}</Text>
                    {done.b ? <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(232, 246, 239)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(35, 155, 110)", textAlign: "center" }}>已完成</Text>
                    </View> : <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(238, 241, 247)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(120, 131, 155)", textAlign: "center" }}>待办</Text>
                    </View>}
                </View>
                <View style={{ flexDirection: "row", spacing: 8 }}>
                    <Button
                        title="完成"
                        onClick={() => done.b = true}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", fontSize: 13 }}
                    />
                    <Button
                        title="撤销"
                        onClick={() => done.b = false}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(232, 236, 244)", color: "rgb(28, 39, 61)", fontSize: 13 }}
                    />
                </View>
            </View>

            {/* 任务 3 */}
            <View style={{
                width: "100%", height: 96, padding: 14, borderRadius: 14, spacing: 10,
                backgroundColor: "rgb(249, 250, 253)",
                borderWidth: 1, borderColor: "rgb(232, 236, 244)",
                flexDirection: "column", justifyContent: "space-between",
            }}>
                <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", spacing: 8 }}>
                    <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)", flexGrow: 1 }}>{titles.c}</Text>
                    {done.c ? <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(232, 246, 239)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(35, 155, 110)", textAlign: "center" }}>已完成</Text>
                    </View> : <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(238, 241, 247)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(120, 131, 155)", textAlign: "center" }}>待办</Text>
                    </View>}
                </View>
                <View style={{ flexDirection: "row", spacing: 8 }}>
                    <Button
                        title="完成"
                        onClick={() => done.c = true}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", fontSize: 13 }}
                    />
                    <Button
                        title="撤销"
                        onClick={() => done.c = false}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(232, 236, 244)", color: "rgb(28, 39, 61)", fontSize: 13 }}
                    />
                </View>
            </View>

            {/* 任务 4 */}
            <View style={{
                width: "100%", height: 96, padding: 14, borderRadius: 14, spacing: 10,
                backgroundColor: "rgb(249, 250, 253)",
                borderWidth: 1, borderColor: "rgb(232, 236, 244)",
                flexDirection: "column", justifyContent: "space-between",
            }}>
                <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", spacing: 8 }}>
                    <Text style={{ fontSize: 15, color: "rgb(28, 39, 61)", flexGrow: 1 }}>{titles.d}</Text>
                    {done.d ? <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(232, 246, 239)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(35, 155, 110)", textAlign: "center" }}>已完成</Text>
                    </View> : <View style={{ width: 56, height: 22, borderRadius: 6, backgroundColor: "rgb(238, 241, 247)",
                    justifyContent: "center" }}>
                        <Text style={{ fontSize: 12, color: "rgb(120, 131, 155)", textAlign: "center" }}>待办</Text>
                    </View>}
                </View>
                <View style={{ flexDirection: "row", spacing: 8 }}>
                    <Button
                        title="完成"
                        onClick={() => done.d = true}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", fontSize: 13 }}
                    />
                    <Button
                        title="撤销"
                        onClick={() => done.d = false}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(232, 236, 244)", color: "rgb(28, 39, 61)", fontSize: 13 }}
                    />
                </View>
            </View>

            {/* 数字状态：番茄计数 */}
            <View style={{
                width: "100%", height: 64, padding: 14, borderRadius: 14,
                backgroundColor: "rgb(240, 237, 255)",
                flexDirection: "row", alignItems: "center", justifyContent: "space-between",
            }}>
                <Text style={{ fontSize: 14, color: "rgb(90, 80, 140)" }}>专注番茄</Text>
                <View style={{ flexDirection: "row", alignItems: "center", spacing: 10 }}>
                    <Button
                        title="-1"
                        onClick={() => pomodoro--}
                        style={{ width: 44, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(120, 92, 220)", color: "white", fontSize: 13 }}
                    />
                    <Text style={{ fontSize: 20, color: "rgb(70, 56, 140)" }}>{pomodoro}</Text>
                    <Button
                        title="+1"
                        onClick={() => pomodoro++}
                        style={{ width: 44, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(120, 92, 220)", color: "white", fontSize: 13 }}
                    />
                </View>
            </View>

            {archived && <View style={{
                width: "100%", height: 88, padding: 14, borderRadius: 14, spacing: 8,
                backgroundColor: "rgb(255, 249, 235)",
                flexDirection: "column", justifyContent: "space-between",
            }}>
                <Text style={{ fontSize: 12, color: "rgb(150, 120, 60)" }}>ARCHIVED</Text>
                <Text style={{ fontSize: 15, color: "rgb(120, 90, 40)" }}>归档的待办会收在这里</Text>
            </View>}

            {/* 条件渲染整块卡片 */}
            <View style={{
                width: "100%", height: 64, padding: 14, borderRadius: 14,
                backgroundColor: "rgb(232, 246, 239)",
                flexDirection: "row", alignItems: "center", justifyContent: "space-between",
            }}>
                <Text style={{ fontSize: 14, color: "rgb(40, 120, 90)" }}>归档区</Text>
                <View style={{ flexDirection: "row", spacing: 8 }}>
                    <Button
                        title="展开"
                        onClick={() => archived = true}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", fontSize: 13 }}
                    />
                    <Button
                        title="收起"
                        onClick={() => archived = false}
                        style={{ width: 72, height: 32, padding: 4, borderRadius: 10, backgroundColor: "white", color: "rgb(40, 120, 90)", fontSize: 13 }}
                    />
                </View>
            </View>

        </ScrollView>

        {/* 浮动底栏：Set 常量归零 */}
        <View style={{
            position: "absolute", left: 16, right: 16, bottom: 10,
            height: 64, padding: 10, flexDirection: "row",
            alignItems: "center", justifyContent: "space-between",
            backgroundColor: "white", borderRadius: 18,
            borderWidth: 1, borderColor: "rgb(54, 65, 82)",
        }}>
            <Text style={{ fontSize: 13, color: "gray" }}>番茄计数可归零</Text>
            <Button
                title="重置"
                onClick={() => pomodoro = 0}
                style={{ width: 100, height: 44, padding: 10, borderRadius: 12, backgroundColor: "rgb(28, 39, 61)", color: "white", fontSize: 15 }}
            />
        </View>
    </View>;
}
