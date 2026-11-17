package com.lumen.app;

import android.app.Activity;
import android.graphics.drawable.GradientDrawable;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

/**
 * 行模板的 Java 侧实现。
 *
 * <p>native 侧只为每个列表解析一次模板（{@link #parse}），之后每建一行只做
 * 「拼一行字段文本 + 一次调用」——建行的 JNI 往返从十几次降到两次，
 * 其余（new TextView、setText、LayoutParams、addView）都在 Java 里完成。
 *
 * <p>模板描述是一行行文本，形如：
 * <pre>
 * L dir=0 grav=16 gap=0 pl=10 pt=0 pr=10 pb=0 ml=0 mt=0 mr=0 mb=8 h=36 bg=-1 rad=6
 * T f=0 w=40 ts=11 col=-12872730 grav=3
 * B a=12 w=44 h=24 ts=11 col=-1328402207 bg=-922367 ts=-1 title=删除
 * C f=2 w=44 ts=11 t=已选 f=未选 tcol=-15591966 fcol=-5000782
 * </pre>
 * 缺省的键取默认值；长度单位一律是 dp（除 ts 是 sp），颜色是 ARGB int。
 */
public final class LumenRow {
    private LumenRow() {}

    /** 一行里所有字段值用这个字符拼接，避免为每个字段单独建 jstring。 */
    public static final String SEP = "\u0001";

    public static final class Slot {
        int kind; // 0 = 字段文本, 1 = 按钮, 2 = 行内条件
        int index; // 字段下标 / 动作下标
        int width = ViewGroup.LayoutParams.WRAP_CONTENT;
        int height = ViewGroup.LayoutParams.WRAP_CONTENT;
        int marginLeft, marginTop, marginRight, marginBottom;
        float textSize;
        int color = 0;
        int gravity = Gravity.LEFT | Gravity.CENTER_VERTICAL;
        int background = 0;
        int radius;
        String title = "";
        String trueText = "";
        String falseText = "";
        int trueColor = 0;
        int falseColor = 0;
    }

    public static final class Template {
        boolean vertical = true;
        boolean spaceBetween;
        int gravity;
        int gap;
        int padLeft, padTop, padRight, padBottom;
        int marginLeft, marginTop, marginRight, marginBottom;
        int height = ViewGroup.LayoutParams.WRAP_CONTENT;
        int width = ViewGroup.LayoutParams.MATCH_PARENT;
        int background = 0;
        int radius;
        Slot[] slots = new Slot[0];
    }

    /** 解析模板描述；每个列表在构建期调用一次。 */
    public static Template parse(String spec) {
        Template template = new Template();
        java.util.ArrayList<Slot> slots = new java.util.ArrayList<>();
        for (String line : spec.split("\n")) {
            if (line.isEmpty()) {
                continue;
            }
            String[] parts = line.split(" ");
            String type = parts[0];
            if ("L".equals(type)) {
                for (int i = 1; i < parts.length; i++) {
                    String[] pair = split(parts[i]);
                    int value = intOf(pair[1], 0);
                    switch (pair[0]) {
                        case "dir": template.vertical = value == 1; break;
                        case "grav": template.gravity = value; break;
                        case "gap": template.gap = value; break;
                        case "pl": template.padLeft = value; break;
                        case "pt": template.padTop = value; break;
                        case "pr": template.padRight = value; break;
                        case "pb": template.padBottom = value; break;
                        case "ml": template.marginLeft = value; break;
                        case "mt": template.marginTop = value; break;
                        case "mr": template.marginRight = value; break;
                        case "mb": template.marginBottom = value; break;
                        case "h": template.height = value; break;
                        case "w": template.width = value; break;
                        case "bg": template.background = value; break;
                        case "rad": template.radius = value; break;
                        case "sb": template.spaceBetween = value == 1; break;
                        default: break;
                    }
                }
                continue;
            }
            Slot slot = new Slot();
            for (int i = 1; i < parts.length; i++) {
                String[] pair = split(parts[i]);
                String raw = pair[1];
                switch (pair[0]) {
                    case "f":
                    case "a": slot.index = intOf(raw, 0); break;
                    case "w": slot.width = intOf(raw, ViewGroup.LayoutParams.WRAP_CONTENT); break;
                    case "h": slot.height = intOf(raw, ViewGroup.LayoutParams.WRAP_CONTENT); break;
                    case "ts": slot.textSize = floatOf(raw, 0f); break;
                    case "col": slot.color = intOf(raw, 0); break;
                    case "grav": slot.gravity = intOf(raw, Gravity.LEFT | Gravity.CENTER_VERTICAL); break;
                    case "bg": slot.background = intOf(raw, 0); break;
                    case "rad": slot.radius = intOf(raw, 0); break;
                    case "ml": slot.marginLeft = intOf(raw, 0); break;
                    case "mt": slot.marginTop = intOf(raw, 0); break;
                    case "mr": slot.marginRight = intOf(raw, 0); break;
                    case "mb": slot.marginBottom = intOf(raw, 0); break;
                    case "title": slot.title = raw; break;
                    case "t": slot.trueText = raw; break;
                    case "ft": slot.falseText = raw; break;
                    case "tcol": slot.trueColor = intOf(raw, 0); break;
                    case "fcol": slot.falseColor = intOf(raw, 0); break;
                    default: break;
                }
            }
            slot.kind = "T".equals(type) ? 0 : ("B".equals(type) ? 1 : 2);
            slots.add(slot);
        }
        template.slots = slots.toArray(new Slot[0]);
        return template;
    }

    /** 拆成 key / raw 两段；没有 `=` 时 raw 为 null。 */
    private static String[] split(String token) {
        int at = token.indexOf('=');
        if (at < 0) {
            return new String[] {token, null};
        }
        return new String[] {token.substring(0, at), token.substring(at + 1)};
    }

    private static int intOf(String raw, int fallback) {
        if (raw == null) {
            return fallback;
        }
        try {
            return Integer.parseInt(raw);
        } catch (NumberFormatException ignored) {
            return fallback;
        }
    }

    private static float floatOf(String raw, float fallback) {
        if (raw == null) {
            return fallback;
        }
        try {
            return Float.parseFloat(raw);
        } catch (NumberFormatException ignored) {
            return fallback;
        }
    }

    /**
     * 建一行并挂到 parent 上；同一个 parent 上已经建过的行会被复用，只改文字和点击 id。
     *
     * @param values 该行所有字段值，用 {@link #SEP} 拼接
     * @param row 行号（按钮点击 id 要用）
     * @param indentPx 层级缩进（像素，已按深度算好）
     */
    public static void build(
            Activity activity,
            Template template,
            ViewGroup parent,
            String values,
            int row,
            int indentPx) {
        String[] cells = values.split(SEP, -1);
        float density = activity.getResources().getDisplayMetrics().density;
        java.util.ArrayList<View> rows = rows(parent);
        if (cursor < rows.size()) {
            LinearLayout reused = (LinearLayout) rows.get(cursor);
            rebind(reused, cells, row);
            applyRowParams(reused, template, density, indentPx);
        } else {
            LinearLayout created = createRow(activity, template, cells, row, density, indentPx);
            rows.add(created);
            parent.addView(created, rowParams(template, density, indentPx));
        }
        cursor++;
    }

    /** 重建开始：把这一轮的复用手标归零。 */
    public static void begin(ViewGroup parent) {
        cursor = 0;
    }

    /** 重建结束：把多出来的行摘掉。 */
    public static void end(ViewGroup parent, int count) {
        java.util.ArrayList<View> rows = rows(parent);
        for (int i = rows.size() - 1; i >= count; i--) {
            parent.removeView(rows.remove(i));
        }
    }

    private static final java.util.WeakHashMap<ViewGroup, java.util.ArrayList<View>> ROWS =
            new java.util.WeakHashMap<>();
    private static int cursor;

    private static java.util.ArrayList<View> rows(ViewGroup parent) {
        java.util.ArrayList<View> rows = ROWS.get(parent);
        if (rows == null) {
            rows = new java.util.ArrayList<>();
            ROWS.put(parent, rows);
        }
        return rows;
    }

    private static LinearLayout createRow(
            Activity activity,
            Template template,
            String[] cells,
            int row,
            float density,
            int indentPx) {
        LinearLayout root = new LinearLayout(activity);
        root.setOrientation(template.vertical ? LinearLayout.VERTICAL : LinearLayout.HORIZONTAL);
        root.setClipChildren(false);
        root.setClipToPadding(false);
        if (template.padLeft + template.padTop + template.padRight + template.padBottom > 0) {
            root.setPadding(
                    px(template.padLeft, density),
                    px(template.padTop, density),
                    px(template.padRight, density),
                    px(template.padBottom, density));
        }
        if (template.background != 0) {
            root.setBackground(background(template.background, template.radius, density));
        }
        if (template.gravity != 0) {
            root.setGravity(template.gravity);
        }
        View[] views = new View[template.slots.length];
        for (int i = 0; i < template.slots.length; i++) {
            views[i] = createChild(activity, template.slots[i], cells, row, density);
            if (i > 0) {
                LinearLayout.LayoutParams params = (LinearLayout.LayoutParams) views[i].getLayoutParams();
                if (template.spaceBetween) {
                    android.widget.Space spacer = new android.widget.Space(activity);
                    root.addView(spacer, new LinearLayout.LayoutParams(0, 0, 1f));
                } else if (template.gap > 0) {
                    params.leftMargin += px(template.gap, density);
                    views[i].setLayoutParams(params);
                }
            }
            root.addView(views[i]);
        }
        root.setTag(new RowState(template.slots, views));
        return root;
    }

    /** 复用行上的局部状态：每个槽位对应的控件，rebind 时直接改它们。 */
    private static final class RowState {
        final Slot[] slots;
        final View[] views;

        RowState(Slot[] slots, View[] views) {
            this.slots = slots;
            this.views = views;
        }
    }

    /** 只改文字和按钮的点击 id，不重建控件。 */
    private static void rebind(LinearLayout row, String[] cells, int rowIndex) {
        RowState state = (RowState) row.getTag();
        if (state == null) {
            return;
        }
        for (int i = 0; i < state.slots.length; i++) {
            Slot slot = state.slots[i];
            View view = state.views[i];
            if (slot.kind == 1) {
                view.setOnClickListener(
                        new MainActivity.ClickBridge(1_000_000 + slot.index * 1024 + rowIndex));
            } else if (slot.kind == 2) {
                boolean on = "true".equals(cell(cells, slot.index));
                ((TextView) view).setText(on ? slot.trueText : slot.falseText);
                int color = on ? slot.trueColor : slot.falseColor;
                if (color != 0) {
                    ((TextView) view).setTextColor(color);
                }
            } else {
                ((TextView) view).setText(cell(cells, slot.index));
            }
        }
    }

    private static LinearLayout.LayoutParams rowParams(
            Template template, float density, int indentPx) {
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
                size(template.width, density, ViewGroup.LayoutParams.MATCH_PARENT),
                size(template.height, density, ViewGroup.LayoutParams.WRAP_CONTENT));
        params.setMargins(
                px(template.marginLeft, density) + indentPx,
                px(template.marginTop, density),
                px(template.marginRight, density),
                px(template.marginBottom, density));
        return params;
    }

    private static void applyRowParams(
            LinearLayout row, Template template, float density, int indentPx) {
        row.setLayoutParams(rowParams(template, density, indentPx));
    }

    private static android.view.View createChild(
            Activity activity, Slot slot, String[] cells, int row, float density) {
        android.view.View view;
        if (slot.kind == 1) {
            Button button = new Button(activity);
            button.setAllCaps(false);
            button.setText(slot.title);
            button.setSingleLine(true);
            button.setIncludeFontPadding(false);
            button.setEllipsize(android.text.TextUtils.TruncateAt.MIDDLE);
            button.setGravity(Gravity.CENTER);
            button.setPadding(0, 0, 0, 0);
            if (slot.color != 0) {
                button.setTextColor(slot.color);
            } else {
                button.setTextColor(0xFFFFFFFF);
            }
            button.setTextSize(1, slot.textSize > 0 ? slot.textSize : 15f);
            if (slot.background != 0) {
                button.setBackground(background(slot.background, slot.radius, density));
            }
            button.setOnClickListener(new MainActivity.ClickBridge(1_000_000 + slot.index * 1024 + row));
            view = button;
        } else {
            TextView text = new TextView(activity);
            String content;
            if (slot.kind == 2) {
                boolean on = "true".equals(cell(cells, slot.index));
                content = on ? slot.trueText : slot.falseText;
                int color = on ? slot.trueColor : slot.falseColor;
                if (color != 0) {
                    text.setTextColor(color);
                }
            } else {
                content = cell(cells, slot.index);
            }
            text.setText(content);
            text.setSingleLine(true);
            text.setIncludeFontPadding(false);
            text.setEllipsize(android.text.TextUtils.TruncateAt.END);
            text.setGravity(slot.gravity);
            if (slot.color != 0) {
                text.setTextColor(slot.color);
            }
            text.setTextSize(1, slot.textSize > 0 ? slot.textSize : 17f);
            view = text;
        }

        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
                size(slot.width, density, ViewGroup.LayoutParams.WRAP_CONTENT),
                size(slot.height, density, ViewGroup.LayoutParams.WRAP_CONTENT));
        params.setMargins(
                px(slot.marginLeft, density),
                px(slot.marginTop, density),
                px(slot.marginRight, density),
                px(slot.marginBottom, density));
        params.gravity = slot.gravity == (Gravity.LEFT | Gravity.CENTER_VERTICAL)
                ? Gravity.CENTER_VERTICAL
                : slot.gravity;
        return applyLayoutParams(view, params);
    }

    private static android.view.View applyLayoutParams(
            android.view.View view, LinearLayout.LayoutParams params) {
        view.setLayoutParams(params);
        return view;
    }

    private static String cell(String[] cells, int index) {
        return index >= 0 && index < cells.length ? cells[index] : "";
    }

    private static int px(int dp, float density) {
        return Math.round(dp * density);
    }

    /** 模板里的尺寸是 dp：正数换算成像素，-1/-2 这类常量透传。 */
    private static int size(int dp, float density, int fallback) {
        if (dp > 0) {
            return Math.round(dp * density);
        }
        if (dp == -1 || dp == -2) {
            return dp;
        }
        return fallback;
    }

    private static GradientDrawable background(int color, int radiusDp, float density) {
        GradientDrawable drawable = new GradientDrawable();
        drawable.setColor(color);
        if (radiusDp > 0) {
            drawable.setCornerRadius(radiusDp * density);
        }
        return drawable;
    }
}
