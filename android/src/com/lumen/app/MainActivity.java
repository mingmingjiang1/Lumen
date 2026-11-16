package com.lumen.app;

import android.app.Activity;
import android.os.Bundle;
import android.text.Editable;
import android.text.TextWatcher;
import android.view.View;
import android.view.WindowInsets;

/**
 * Android 宿主壳：只负责加载 native 库，把 Activity 交给编译器生成的界面代码。
 * 界面本身由 liblumen.so 通过 JNI 创建（cargo run -- android 生成 out.ll）。
 *
 * 四个桥接类把 Java 侧事件转成 native 回调，动作下标由编译器在创建控件时写入。
 */
public class MainActivity extends Activity {
    static {
        System.loadLibrary("lumen");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        applySystemBarInset();
        nativeCreate(this);
    }

    /**
     * targetSdk 35 起系统强制 edge-to-edge，内容会画到状态栏下面。
     * 这里把状态栏高度作为内边距补到内容视图上，让界面从状态栏下方开始。
     * 先按资源里的大小同步设一次（不依赖 inset 回调的时机），回调到了再修正。
     */
    @SuppressWarnings("deprecation")
    private void applySystemBarInset() {
        View content = getWindow().getDecorView().findViewById(android.R.id.content);
        content.setPadding(0, statusBarHeight(), 0, 0);
        content.setOnApplyWindowInsetsListener((view, insets) -> {
            int top = insets.getSystemWindowInsetTop();
            if (view.getPaddingTop() != top) {
                view.setPadding(view.getPaddingLeft(), top, view.getPaddingRight(), view.getPaddingBottom());
            }
            return insets;
        });
    }

    private int statusBarHeight() {
        int id = getResources().getIdentifier("status_bar_height", "dimen", "android");
        return id > 0 ? getResources().getDimensionPixelSize(id) : 0;
    }

    private native void nativeCreate(Activity activity);

    private static native void nativeOnClick(int id);

    private static native void nativeOnInput(int id, String text);

    private static native void nativeOnFocus(int id);

    private static native void nativeOnScroll(int scrollY);

    /** 按钮点击 -> 状态动作。 */
    public static final class ClickBridge implements View.OnClickListener {
        private final int id;

        public ClickBridge(int id) {
            this.id = id;
        }

        @Override
        public void onClick(View view) {
            nativeOnClick(id);
        }
    }

    /** 输入框文本变化 -> 把文本写进字符串状态单元。 */
    public static final class InputBridge implements TextWatcher {
        private final int id;

        public InputBridge(int id) {
            this.id = id;
        }

        @Override
        public void beforeTextChanged(CharSequence text, int start, int count, int after) {}

        @Override
        public void onTextChanged(CharSequence text, int start, int before, int count) {}

        @Override
        public void afterTextChanged(Editable editable) {
            nativeOnInput(id, editable.toString());
        }
    }

    /** 输入框获得焦点 -> 布尔状态置真。 */
    public static final class FocusBridge implements View.OnFocusChangeListener {
        private final int id;

        public FocusBridge(int id) {
            this.id = id;
        }

        @Override
        public void onFocusChange(View view, boolean hasFocus) {
            if (hasFocus) {
                nativeOnFocus(id);
            }
        }
    }

    /** 滚动 -> 顶部栏渐显。 */
    public static final class ScrollBridge implements View.OnScrollChangeListener {
        @SuppressWarnings("unused")
        private final int id;

        public ScrollBridge(int id) {
            this.id = id;
        }

        @Override
        public void onScrollChange(View view, int scrollX, int scrollY, int oldScrollX, int oldScrollY) {
            nativeOnScroll(scrollY);
        }
    }
}
