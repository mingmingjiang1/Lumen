/**
 * Lumen 示例页面：src/main.rs 用 include_str! 读它并编译成 UIKit 代码。
 * 同目录的 lumen.d.ts 提供 JSX / state 的类型声明，仓库根目录执行
 *   npx -p typescript tsc -p app
 * 可做类型检查。
 */

function InsightCard({ title, height }: { title: string; height: number }) {
    onMount(() => {});
    onUnmount(() => {});
    return <View style={{
        width: "100%", height, padding: 18,
        flexDirection: "column", justifyContent: "space-between",
        backgroundColor: "rgb(235, 241, 252)", borderRadius: 16
    }}>
        <Text style={{ fontSize: 14, color: "gray" }}>CONDITIONAL COMPONENT</Text>
        <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>{title}</Text>
    </View>;
}

function App() {
    let showInsight = state(false);
    // 复杂状态：对象会被展开成多个独立单元，name 和 visits 各自更新。
    const profile = state({ name: "guest", visits: 0 });
    // 滚动进度：以前是内建的 fadeTarget，现在用 onScroll + clamp + opacity 自己写
    const progress = state(0);
    return <View style={{
            width: "100%", height: "100%",
            backgroundColor: "rgb(242, 245, 250)",
            flexDirection: "column",
        }}>
            <View style={{
                id: "topBar",
                ignoresSafeArea: "top",
                opacity: progress,
                width: "100%", height: 44, padding: 12,
                flexDirection: "row", alignItems: "center", justifyContent: "space-between",
                backgroundColor: "rgb(28, 39, 61)"
            }}>
                <Text style={{ fontSize: 22, color: "white", textAlign: "center" }}>My Dashboard</Text>
                <Text style={{ fontSize: 14, color: "rgb(190, 205, 230)", textAlign: "center" }}>TODAY</Text>
            </View>
            <ScrollView onScroll={(y) => progress = clamp(y, 0, 100)} style={{
                marginHorizontal: 16, flexGrow: 1, marginTop: 10,
                marginBottom: 80,
                padding: 16, spacing: 14, backgroundColor: "white", borderRadius: 18
            }}>
                <Text style={{ fontSize: 26, color: "rgb(28, 39, 61)" }}>Reactive controls</Text>
                <Text style={{ fontSize: 16, color: "gray" }}>Toggle a component to see reactive conditional rendering.</Text>
                <View style={{
                    width: "100%", height: 120, padding: 18,
                    flexDirection: "column", justifyContent: "space-between",
                    backgroundColor: "rgb(69, 104, 220)", borderRadius: 16
                }}>
                    <Text style={{ fontSize: 14, color: "rgb(220, 230, 255)" }}>CONDITIONAL STATE</Text>
                    <Text style={{ fontSize: 22, color: "white" }}>{showInsight}</Text>
                </View>
                <Text style={{ fontSize: 18, color: "rgb(28, 39, 61)" }}>Conditional component</Text>
                <View style={{
                    width: "100%", height: 54, flexDirection: "row",
                    alignItems: "center", justifyContent: "space-between", spacing: 10
                }}>
                    <Button
                        title="Show component"
                        onClick={() => showInsight = true}
                        style={{ width: 140, height: 48, padding: 10, backgroundColor: "rgb(69, 104, 220)", color: "white", borderRadius: 12 }}
                    />
                    <Button
                        title="Remove component"
                        onClick={() => showInsight = false}
                        style={{ width: 140, height: 48, padding: 10, backgroundColor: "rgb(35, 155, 110)", color: "white", borderRadius: 12 }}
                    />
                </View>
                {showInsight && <InsightCard title="Props, lifecycle, and real view mounting" height={150} />}
                <Text style={{ fontSize: 14, color: "gray" }}>The card receives scalar props and runs mount/unmount hooks as it is added or removed.</Text>
                <Text style={{ fontSize: 18, color: "rgb(28, 39, 61)" }}>Complex state</Text>
                <View style={{
                    width: "100%", height: 120, padding: 18,
                    flexDirection: "column", justifyContent: "space-between",
                    backgroundColor: "rgb(240, 237, 255)", borderRadius: 16
                }}>
                    <Text style={{ fontSize: 14, color: "gray" }}>PROFILE</Text>
                    <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>{profile.name}</Text>
                    <Text style={{ fontSize: 16, color: "gray" }}>{profile.visits}</Text>
                </View>
                <View style={{
                    width: "100%", height: 54, flexDirection: "row",
                    alignItems: "center", justifyContent: "space-between", spacing: 10
                }}>
                    <Button
                        title="Visit +1"
                        onClick={() => profile.visits++}
                        style={{ width: 140, height: 48, padding: 10, backgroundColor: "rgb(120, 92, 220)", color: "white", borderRadius: 12 }}
                    />
                    <Button
                        title="Rename"
                        onClick={() => profile.name = "Ada"}
                        style={{ width: 140, height: 48, padding: 10, backgroundColor: "rgb(28, 39, 61)", color: "white", borderRadius: 12 }}
                    />
                </View>
                <Text style={{ fontSize: 14, color: "gray" }}>One object state, one cell per field: name and visits update independently.</Text>
                <Text style={{ fontSize: 18, color: "rgb(28, 39, 61)" }}>More to explore</Text>
                <View style={{
                    width: "100%", height: 190, padding: 18,
                    flexDirection: "column", justifyContent: "space-between",
                    backgroundColor: "rgb(235, 241, 252)", borderRadius: 16
                }}>
                    <Text style={{ fontSize: 14, color: "gray" }}>DAILY MOMENTUM</Text>
                    <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>Small steps build lasting progress.</Text>
                </View>
                <View style={{
                    width: "100%", height: 190, padding: 18,
                    flexDirection: "column", justifyContent: "space-between",
                    backgroundColor: "rgb(232, 246, 239)", borderRadius: 16
                }}>
                    <Text style={{ fontSize: 14, color: "gray" }}>NEXT UP</Text>
                    <Text style={{ fontSize: 22, color: "rgb(28, 39, 61)" }}>Keep your streak going tomorrow.</Text>
                </View>
            </ScrollView>
            <View style={{
                position: "absolute", left: 16, right: 16, bottom: 10,
                height: 64, padding: 10, flexDirection: "row",
                alignItems: "center", justifyContent: "space-between",
                backgroundColor: "white", borderRadius: 18,
                borderWidth: 1, borderColor: "rgb(54, 65, 82)"
            }}>
                <Text style={{ fontSize: 14, color: "gray" }}>One signal, updated in place</Text>
                <Button
                    title="Hide"
                    onClick={() => showInsight = false}
                    style={{ width: 100, height: 44, backgroundColor: "rgb(28, 39, 61)", color: "white", borderRadius: 12 }}
                />
            </View>
        </View>
}
