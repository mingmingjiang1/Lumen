/**
 * Lumen TSX 的类型声明：只描述编译器 src/main.rs 实际支持的那个子集，
 * 供编辑器和 `tsc` 做类型检查。这些声明不参与编译，运行时行为由 codegen 决定。
 *
 * 注意：JSX 里大写开头的标签会被 TypeScript 当作「组件值」而不是 intrinsic
 * element，所以这里用 `declare function` 声明 View / Text 等，而不是列在
 * JSX.IntrinsicElements 里。
 */

declare namespace JSX {
  interface Element {}
}

type LumenDimension = number | `${number}%`;

interface LumenStyle {
  width?: LumenDimension;
  height?: LumenDimension;

  padding?: number;
  paddingHorizontal?: number;
  paddingVertical?: number;
  paddingTop?: number;
  paddingRight?: number;
  paddingBottom?: number;
  paddingLeft?: number;

  margin?: number;
  marginHorizontal?: number;
  marginVertical?: number;
  marginTop?: number;
  marginRight?: number;
  marginBottom?: number;
  marginLeft?: number;

  position?: "relative" | "absolute";
  top?: LumenDimension;
  right?: LumenDimension;
  bottom?: LumenDimension;
  left?: LumenDimension;

  spacing?: number;
  gap?: number;
  flexDirection?: "row" | "column";
  justifyContent?: "start" | "center" | "end" | "space-between" | "space-around";
  alignItems?: "start" | "center" | "end" | "stretch";
  flexGrow?: number;

  fontSize?: number;
  textAlign?: "left" | "center" | "right";
  color?: string;
  backgroundColor?: string;
  /** Input 占位符颜色；不写时三端统一用同一个中性灰。 */
  placeholderColor?: string;

  borderWidth?: number;
  borderColor?: string;
  borderRadius?: number;

  /** 视图标识，供 fadeTarget 引用。 */
  id?: string;
  /** 让背景继续向上铺到父视图顶部，只影响绘制，不改变布局。 */
  backgroundExtendsTo?: "top";
  /** 让视图上边缘贴到父视图物理顶部，高度自动加上安全区（对齐 SwiftUI 的 ignoresSafeArea）。 */
  ignoresSafeArea?: "top";

  /** 滚动容器的淡出配置：滚动时把 id 对应的视图淡入。 */
  fadeTarget?: string;
  fadeStart?: number;
  fadeDistance?: number;
}

type LumenChild = JSX.Element | string | number | boolean | null | undefined;

interface LumenCommonProps {
  style?: LumenStyle;
  children?: LumenChild;
}

interface LumenButtonProps extends LumenCommonProps {
  title?: string;
  onClick?: () => void;
}

interface LumenInputProps extends LumenCommonProps {
  placeholder?: string;
  onInput?: (value: string) => void;
  onFocus?: () => void;
}

interface LumenImageProps extends LumenCommonProps {
  src?: string;
  source?: string;
  mode?: "scaleToFill" | "aspectFit" | "aspectFill" | "widthFix" | "heightFix";
}

declare function View(props: LumenCommonProps): JSX.Element;
declare function VStack(props: LumenCommonProps): JSX.Element;
declare function ScrollView(props: LumenCommonProps): JSX.Element;
declare function Text(props: LumenCommonProps): JSX.Element;
declare function Button(props: LumenButtonProps): JSX.Element;
declare function Input(props: LumenInputProps): JSX.Element;
declare function Image(props: LumenImageProps): JSX.Element;

/** 声明一个标量状态；读它得到当前值，直接赋值即可更新引用它的视图。 */
declare function state(initial: boolean): boolean;
declare function state(initial: number): number;
declare function state(initial: string): string;
/** 对象状态会被展开成多个独立单元：读写 `profile.visits` 只更新引用该字段的视图。 */
declare function state<T extends Record<string, number | string | boolean>>(initial: T): T;
declare function onMount(effect: () => void): void;
declare function onUnmount(effect: () => void): void;
