#include <jni.h>
#include <stdio.h>

static JavaVM *g_vm;
static jobject g_counter;

static jobject call_view(jmethodID mid) { (void)mid; return NULL; }

static jobject new_instance(JNIEnv *env, jobject ctx, const char *cls_name) {
    jclass cls = (*env)->FindClass(env, cls_name);
    jmethodID ctor = (*env)->GetMethodID(env, cls, "<init>", "(Landroid/content/Context;)V");
    return (*env)->NewObject(env, cls, ctor, ctx);
}

static void set_text(JNIEnv *env, jobject view, const char *cls_name, const char *text) {
    jclass cls = (*env)->FindClass(env, cls_name);
    jstring value = (*env)->NewStringUTF(env, text);
    jmethodID set_text = (*env)->GetMethodID(env, cls, "setText", "(Ljava/lang/CharSequence;)V");
    (*env)->CallVoidMethod(env, view, set_text, value);
    (*env)->DeleteLocalRef(env, value);
}

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM *vm, void *reserved) {
    g_vm = vm;
    return JNI_VERSION_1_6;
}

JNIEXPORT void JNICALL
Java_com_lumen_app_MainActivity_nativeCreate(JNIEnv *env, jobject thiz, jobject activity) {
    jclass ll_cls = (*env)->FindClass(env, "android/widget/LinearLayout");
    jmethodID ll_ctor = (*env)->GetMethodID(env, ll_cls, "<init>", "(Landroid/content/Context;)V");
    jobject root = (*env)->NewObject(env, ll_cls, ll_ctor, activity);
    jmethodID set_orientation = (*env)->GetMethodID(env, ll_cls, "setOrientation", "(I)V");
    (*env)->CallVoidMethod(env, root, set_orientation, 1 /* VERTICAL */);
    jmethodID set_padding = (*env)->GetMethodID(env, ll_cls, "setPadding", "(IIII)V");
    (*env)->CallVoidMethod(env, root, set_padding, 64, 96, 64, 64);

    jobject title = new_instance(env, activity, "android/widget/TextView");
    set_text(env, title, "android/widget/TextView", "Lumen on Android");
    jclass tv_cls = (*env)->FindClass(env, "android/widget/TextView");
    jmethodID set_size = (*env)->GetMethodID(env, tv_cls, "setTextSize", "(F)V");
    (*env)->CallVoidMethod(env, title, set_size, 26.0f);

    jobject counter = new_instance(env, activity, "android/widget/TextView");
    set_text(env, counter, "android/widget/TextView", "0");
    (*env)->CallVoidMethod(env, counter, set_size, 40.0f);
    g_counter = (*env)->NewGlobalRef(env, counter);

    jobject button = new_instance(env, activity, "android/widget/Button");
    set_text(env, button, "android/widget/Button", "Visit +1");
    jclass bridge_cls = (*env)->FindClass(env, "com/lumen/app/MainActivity$ClickBridge");
    jmethodID bridge_ctor = (*env)->GetMethodID(env, bridge_cls, "<init>", "(I)V");
    jobject bridge = (*env)->NewObject(env, bridge_cls, bridge_ctor, 1);
    jclass btn_cls = (*env)->FindClass(env, "android/widget/Button");
    jmethodID set_listener =
        (*env)->GetMethodID(env, btn_cls, "setOnClickListener", "(Landroid/view/View$OnClickListener;)V");
    (*env)->CallVoidMethod(env, button, set_listener, bridge);

    jclass ll_cls2 = (*env)->FindClass(env, "android/widget/LinearLayout");
    jmethodID add_view = (*env)->GetMethodID(env, ll_cls2, "addView", "(Landroid/view/View;)V");
    (*env)->CallVoidMethod(env, root, add_view, title);
    (*env)->CallVoidMethod(env, root, add_view, counter);
    (*env)->CallVoidMethod(env, root, add_view, button);

    jclass act_cls = (*env)->GetObjectClass(env, activity);
    jmethodID set_content = (*env)->GetMethodID(env, act_cls, "setContentView", "(Landroid/view/View;)V");
    (*env)->CallVoidMethod(env, activity, set_content, root);
    (void)call_view;
}

JNIEXPORT void JNICALL
Java_com_lumen_app_MainActivity_nativeOnClick(JNIEnv *env, jclass klass, jint id) {
    static int visits = 0;
    char buffer[32];
    visits += 1;
    snprintf(buffer, sizeof(buffer), "%d", visits);
    set_text(env, g_counter, "android/widget/TextView", buffer);
    (void)id;
    (void)klass;
}
