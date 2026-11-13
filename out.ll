; ModuleID = 'ios_app'
source_filename = "ios_app"
target triple = "arm64-apple-ios"

@g_root_view = global ptr null
@g_state_value = global ptr null
@g_scroll_fade_view = global ptr null
@g_scroll_fade_scroll = global ptr null
@g_top_background_view = global ptr null
@g_scroll_fade_background = global ptr null
@class_name = private unnamed_addr constant [9 x i8] c"NSObject\00", align 1
@app_delegate_class_name = private unnamed_addr constant [12 x i8] c"AppDelegate\00", align 1
@text_content = private unnamed_addr constant [6 x i8] c"false\00", align 1
@class_name.1 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.2 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@class_name.3 = private unnamed_addr constant [12 x i8] c"UIStackView\00", align 1
@sel_name.4 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.5 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.6 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.7 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.8 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.9 = private unnamed_addr constant [9 x i8] c"setAxis:\00", align 1
@sel_name.10 = private unnamed_addr constant [12 x i8] c"setSpacing:\00", align 1
@sel_name.11 = private unnamed_addr constant [14 x i8] c"setAlignment:\00", align 1
@sel_name.12 = private unnamed_addr constant [17 x i8] c"setDistribution:\00", align 1
@sel_name.13 = private unnamed_addr constant [37 x i8] c"setLayoutMarginsRelativeArrangement:\00", align 1
@sel_name.14 = private unnamed_addr constant [18 x i8] c"setLayoutMargins:\00", align 1
@class_name.15 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.16 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.17 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.18 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.19 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.20 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.21 = private unnamed_addr constant [10 x i8] c"setAlpha:\00", align 1
@class_name.22 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.23 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.24 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.25 = private unnamed_addr constant [13 x i8] c"My Dashboard\00", align 1
@class_name.26 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.27 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.28 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.29 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.30 = private unnamed_addr constant [18 x i8] c"setTextAlignment:\00", align 1
@class_name.31 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.32 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.33 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.34 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.35 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.36 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.37 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.38 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.39 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.40 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.41 = private unnamed_addr constant [6 x i8] c"TODAY\00", align 1
@class_name.42 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.43 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.44 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.45 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.46 = private unnamed_addr constant [18 x i8] c"setTextAlignment:\00", align 1
@class_name.47 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.48 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.49 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.50 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.51 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.52 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.53 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.54 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.55 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.56 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.57 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.58 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.59 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.60 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.61 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.62 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.63 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.64 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.65 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.66 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.67 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.68 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.69 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.70 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.71 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.72 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.73 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.74 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.75 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.76 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.77 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.78 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.79 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.80 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.81 = private unnamed_addr constant [13 x i8] c"UIScrollView\00", align 1
@sel_name.82 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.83 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.84 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.85 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.86 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.87 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.88 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@sel_name.89 = private unnamed_addr constant [13 x i8] c"setDelegate:\00", align 1
@class_name.90 = private unnamed_addr constant [12 x i8] c"UIStackView\00", align 1
@sel_name.91 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.92 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@sel_name.93 = private unnamed_addr constant [9 x i8] c"setAxis:\00", align 1
@sel_name.94 = private unnamed_addr constant [12 x i8] c"setSpacing:\00", align 1
@sel_name.95 = private unnamed_addr constant [14 x i8] c"setAlignment:\00", align 1
@sel_name.96 = private unnamed_addr constant [17 x i8] c"setDistribution:\00", align 1
@sel_name.97 = private unnamed_addr constant [37 x i8] c"setLayoutMarginsRelativeArrangement:\00", align 1
@sel_name.98 = private unnamed_addr constant [18 x i8] c"setLayoutMargins:\00", align 1
@sel_name.99 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.100 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.101 = private unnamed_addr constant [19 x i8] c"contentLayoutGuide\00", align 1
@sel_name.102 = private unnamed_addr constant [17 x i8] c"frameLayoutGuide\00", align 1
@sel_name.103 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.104 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.105 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.106 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.107 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.108 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.109 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.110 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.111 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.112 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.113 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.114 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.115 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.116 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.117 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.118 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.119 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.120 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.121 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.122 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.123 = private unnamed_addr constant [9 x i8] c"setAxis:\00", align 1
@sel_name.124 = private unnamed_addr constant [12 x i8] c"setSpacing:\00", align 1
@sel_name.125 = private unnamed_addr constant [14 x i8] c"setAlignment:\00", align 1
@sel_name.126 = private unnamed_addr constant [17 x i8] c"setDistribution:\00", align 1
@sel_name.127 = private unnamed_addr constant [37 x i8] c"setLayoutMarginsRelativeArrangement:\00", align 1
@sel_name.128 = private unnamed_addr constant [18 x i8] c"setLayoutMargins:\00", align 1
@class_name.129 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.130 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.131 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.132 = private unnamed_addr constant [18 x i8] c"Reactive controls\00", align 1
@class_name.133 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.134 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.135 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.136 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.137 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.138 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.139 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.140 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.141 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.142 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.143 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.144 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@class_name.145 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.146 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.147 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.148 = private unnamed_addr constant [58 x i8] c"Toggle a component to see reactive conditional rendering.\00", align 1
@class_name.149 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.150 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.151 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.152 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.153 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.154 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.155 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.156 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.157 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.158 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.159 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.160 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@class_name.161 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.162 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.163 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.164 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.165 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.166 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.167 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.168 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.169 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.170 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.171 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.172 = private unnamed_addr constant [18 x i8] c"CONDITIONAL STATE\00", align 1
@class_name.173 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.174 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.175 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.176 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.177 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.178 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.179 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.180 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.181 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.182 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.183 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.184 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.185 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.186 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.187 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@class_name.188 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.189 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.190 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.191 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.192 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.193 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.194 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.195 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.196 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.197 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@g_state_label_0 = global ptr null
@sel_name.198 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.199 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.200 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.201 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.202 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.203 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.204 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.205 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.206 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.207 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.208 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.209 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.210 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.211 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.212 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.213 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.214 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.215 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.216 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.217 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.218 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.219 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.220 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.221 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.222 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.223 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.224 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.225 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.226 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.227 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.228 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.229 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.230 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.231 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.232 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.233 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.234 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.235 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.236 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.237 = private unnamed_addr constant [22 x i8] c"Conditional component\00", align 1
@class_name.238 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.239 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.240 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.241 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.242 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.243 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.244 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.245 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.246 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.247 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.248 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.249 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@class_name.250 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.251 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.252 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.253 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.254 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.255 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.256 = private unnamed_addr constant [9 x i8] c"UIButton\00", align 1
@sel_name.257 = private unnamed_addr constant [16 x i8] c"buttonWithType:\00", align 1
@sel_name.258 = private unnamed_addr constant [10 x i8] c"setFrame:\00", align 1
@text_content.259 = private unnamed_addr constant [15 x i8] c"Show component\00", align 1
@class_name.260 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.261 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.262 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.263 = private unnamed_addr constant [19 x i8] c"setTitle:forState:\00", align 1
@class_name.264 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.265 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.266 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.267 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.268 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.269 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.270 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.271 = private unnamed_addr constant [24 x i8] c"setTitleColor:forState:\00", align 1
@sel_name.272 = private unnamed_addr constant [14 x i8] c"stateAction0:\00", align 1
@sel_name.273 = private unnamed_addr constant [35 x i8] c"addTarget:action:forControlEvents:\00", align 1
@sel_name.274 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.275 = private unnamed_addr constant [9 x i8] c"UIButton\00", align 1
@sel_name.276 = private unnamed_addr constant [16 x i8] c"buttonWithType:\00", align 1
@sel_name.277 = private unnamed_addr constant [10 x i8] c"setFrame:\00", align 1
@text_content.278 = private unnamed_addr constant [17 x i8] c"Remove component\00", align 1
@class_name.279 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.280 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.281 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.282 = private unnamed_addr constant [19 x i8] c"setTitle:forState:\00", align 1
@class_name.283 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.284 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.285 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.286 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.287 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.288 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.289 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.290 = private unnamed_addr constant [24 x i8] c"setTitleColor:forState:\00", align 1
@sel_name.291 = private unnamed_addr constant [14 x i8] c"stateAction1:\00", align 1
@sel_name.292 = private unnamed_addr constant [35 x i8] c"addTarget:action:forControlEvents:\00", align 1
@sel_name.293 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.294 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.295 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.296 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.297 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.298 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.299 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.300 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.301 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.302 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.303 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.304 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.305 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.306 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.307 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.308 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.309 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.310 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.311 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.312 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.313 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.314 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.315 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.316 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.317 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.318 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.319 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.320 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.321 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.322 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.323 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.324 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.325 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.326 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.327 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.328 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.329 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.330 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.331 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.332 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.333 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.334 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.335 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.336 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.337 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.338 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.339 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.340 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.341 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.342 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.343 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.344 = private unnamed_addr constant [22 x i8] c"CONDITIONAL COMPONENT\00", align 1
@class_name.345 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.346 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.347 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.348 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.349 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.350 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.351 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.352 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.353 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.354 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.355 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.356 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.357 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.358 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.359 = private unnamed_addr constant [41 x i8] c"Props, lifecycle, and real view mounting\00", align 1
@class_name.360 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.361 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.362 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.363 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.364 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.365 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.366 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.367 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.368 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.369 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.370 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.371 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.372 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.373 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.374 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.375 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.376 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.377 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.378 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.379 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.380 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.381 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.382 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.383 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.384 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.385 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.386 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.387 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.388 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.389 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.390 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.391 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.392 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.393 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.394 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.395 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.396 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.397 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@g_conditional_true_0 = global ptr null
@g_conditional_false_0 = global ptr null
@g_conditional_stack_0 = global ptr null
@g_conditional_state_0 = global i8 0
@sel_name.398 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.399 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.400 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.401 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.402 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.403 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.404 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.405 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.406 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.407 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.408 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.409 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.410 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.411 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.412 = private unnamed_addr constant [87 x i8] c"The card receives scalar props and runs mount/unmount hooks as it is added or removed.\00", align 1
@class_name.413 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.414 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.415 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.416 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.417 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.418 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.419 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.420 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.421 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.422 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.423 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.424 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@class_name.425 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.426 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.427 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.428 = private unnamed_addr constant [16 x i8] c"More to explore\00", align 1
@class_name.429 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.430 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.431 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.432 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.433 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.434 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.435 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.436 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.437 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.438 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.439 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.440 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@class_name.441 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.442 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.443 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.444 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.445 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.446 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.447 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.448 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.449 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.450 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.451 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.452 = private unnamed_addr constant [15 x i8] c"DAILY MOMENTUM\00", align 1
@class_name.453 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.454 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.455 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.456 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.457 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.458 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.459 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.460 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.461 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.462 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.463 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.464 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.465 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.466 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.467 = private unnamed_addr constant [36 x i8] c"Small steps build lasting progress.\00", align 1
@class_name.468 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.469 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.470 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.471 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.472 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.473 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.474 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.475 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.476 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.477 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.478 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.479 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.480 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.481 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.482 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.483 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.484 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.485 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.486 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.487 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.488 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.489 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.490 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.491 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.492 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.493 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.494 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.495 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.496 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.497 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.498 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.499 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.500 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.501 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.502 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.503 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.504 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.505 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.506 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.507 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.508 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.509 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.510 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.511 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.512 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.513 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.514 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.515 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.516 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.517 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.518 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.519 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.520 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.521 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.522 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.523 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.524 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.525 = private unnamed_addr constant [8 x i8] c"NEXT UP\00", align 1
@class_name.526 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.527 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.528 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.529 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.530 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.531 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.532 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.533 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.534 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.535 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.536 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.537 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.538 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.539 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.540 = private unnamed_addr constant [33 x i8] c"Keep your streak going tomorrow.\00", align 1
@class_name.541 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.542 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.543 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.544 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.545 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.546 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.547 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.548 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.549 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.550 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.551 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.552 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.553 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.554 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.555 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.556 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.557 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.558 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.559 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.560 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.561 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.562 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.563 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.564 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.565 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.566 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.567 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.568 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.569 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.570 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.571 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.572 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.573 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.574 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.575 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.576 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.577 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.578 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.579 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.580 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.581 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.582 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.583 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.584 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.585 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.586 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.587 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.588 = private unnamed_addr constant [20 x i8] c"addArrangedSubview:\00", align 1
@sel_name.589 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.590 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.591 = private unnamed_addr constant [45 x i8] c"constraintEqualToAnchor:multiplier:constant:\00", align 1
@sel_name.592 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.593 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.594 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.595 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.596 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.597 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.598 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.599 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.600 = private unnamed_addr constant [16 x i8] c"setBorderWidth:\00", align 1
@class_name.601 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.602 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.603 = private unnamed_addr constant [8 x i8] c"CGColor\00", align 1
@sel_name.604 = private unnamed_addr constant [16 x i8] c"setBorderColor:\00", align 1
@sel_name.605 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.606 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.607 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.608 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.609 = private unnamed_addr constant [29 x i8] c"One signal, updated in place\00", align 1
@class_name.610 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.611 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.612 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.613 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@class_name.614 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.615 = private unnamed_addr constant [10 x i8] c"grayColor\00", align 1
@sel_name.616 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.617 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.618 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.619 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.620 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.621 = private unnamed_addr constant [9 x i8] c"UIButton\00", align 1
@sel_name.622 = private unnamed_addr constant [16 x i8] c"buttonWithType:\00", align 1
@sel_name.623 = private unnamed_addr constant [10 x i8] c"setFrame:\00", align 1
@text_content.624 = private unnamed_addr constant [5 x i8] c"Hide\00", align 1
@class_name.625 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.626 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.627 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.628 = private unnamed_addr constant [19 x i8] c"setTitle:forState:\00", align 1
@class_name.629 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.630 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.631 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.632 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.633 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@class_name.634 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.635 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.636 = private unnamed_addr constant [24 x i8] c"setTitleColor:forState:\00", align 1
@sel_name.637 = private unnamed_addr constant [14 x i8] c"stateAction4:\00", align 1
@sel_name.638 = private unnamed_addr constant [35 x i8] c"addTarget:action:forControlEvents:\00", align 1
@sel_name.639 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.640 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.641 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.642 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.643 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.644 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.645 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.646 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.647 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.648 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.649 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.650 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.651 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.652 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.653 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.654 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.655 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.656 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.657 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.658 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.659 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.660 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.661 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.662 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.663 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.664 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.665 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.666 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.667 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.668 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.669 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.670 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.671 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.672 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.673 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.674 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.675 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.676 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.677 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.678 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.679 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.680 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.681 = private unnamed_addr constant [9 x i8] c"UIScreen\00", align 1
@sel_name.682 = private unnamed_addr constant [11 x i8] c"mainScreen\00", align 1
@sel_name.683 = private unnamed_addr constant [7 x i8] c"bounds\00", align 1
@class_name.684 = private unnamed_addr constant [9 x i8] c"UIWindow\00", align 1
@sel_name.685 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.686 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.687 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.688 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.689 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.690 = private unnamed_addr constant [17 x i8] c"UIViewController\00", align 1
@sel_name.691 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.692 = private unnamed_addr constant [5 x i8] c"init\00", align 1
@sel_name.693 = private unnamed_addr constant [5 x i8] c"view\00", align 1
@sel_name.694 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.695 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.696 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.697 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.698 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.699 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.700 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.701 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.702 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.703 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.704 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.705 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.706 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.707 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.708 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.709 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.710 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.711 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.712 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.713 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.714 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.715 = private unnamed_addr constant [5 x i8] c"init\00", align 1
@class_name.716 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.717 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.718 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.719 = private unnamed_addr constant [27 x i8] c"setUserInteractionEnabled:\00", align 1
@sel_name.720 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.721 = private unnamed_addr constant [23 x i8] c"insertSubview:atIndex:\00", align 1
@sel_name.722 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.723 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.724 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.725 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.726 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.727 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.728 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.729 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.730 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.731 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.732 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.733 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.734 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.735 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.736 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.737 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.738 = private unnamed_addr constant [10 x i8] c"setAlpha:\00", align 1
@sel_name.739 = private unnamed_addr constant [23 x i8] c"setRootViewController:\00", align 1
@sel_name.740 = private unnamed_addr constant [18 x i8] c"makeKeyAndVisible\00", align 1
@sel_name.741 = private unnamed_addr constant [43 x i8] c"application:didFinishLaunchingWithOptions:\00", align 1
@method_type_encoding = private unnamed_addr constant [6 x i8] c"c@:@@\00", align 1
@text_content.742 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@class_name.743 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.744 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.745 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.746 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.747 = private unnamed_addr constant [10 x i8] c"boolValue\00", align 1
@sel_name.748 = private unnamed_addr constant [23 x i8] c"removeArrangedSubview:\00", align 1
@sel_name.749 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.750 = private unnamed_addr constant [20 x i8] c"removeFromSuperview\00", align 1
@sel_name.751 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.752 = private unnamed_addr constant [31 x i8] c"insertArrangedSubview:atIndex:\00", align 1
@sel_name.753 = private unnamed_addr constant [15 x i8] c"layoutIfNeeded\00", align 1
@sel_name.754 = private unnamed_addr constant [14 x i8] c"stateAction3:\00", align 1
@sel_name.755 = private unnamed_addr constant [14 x i8] c"stateAction2:\00", align 1
@sel_name.756 = private unnamed_addr constant [14 x i8] c"stateAction0:\00", align 1
@state_action_0_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@text_content.757 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@class_name.758 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.759 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.760 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.761 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.762 = private unnamed_addr constant [10 x i8] c"boolValue\00", align 1
@sel_name.763 = private unnamed_addr constant [23 x i8] c"removeArrangedSubview:\00", align 1
@sel_name.764 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.765 = private unnamed_addr constant [20 x i8] c"removeFromSuperview\00", align 1
@sel_name.766 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.767 = private unnamed_addr constant [31 x i8] c"insertArrangedSubview:atIndex:\00", align 1
@sel_name.768 = private unnamed_addr constant [15 x i8] c"layoutIfNeeded\00", align 1
@sel_name.769 = private unnamed_addr constant [14 x i8] c"stateAction3:\00", align 1
@sel_name.770 = private unnamed_addr constant [14 x i8] c"stateAction2:\00", align 1
@sel_name.771 = private unnamed_addr constant [14 x i8] c"stateAction1:\00", align 1
@state_action_1_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@sel_name.772 = private unnamed_addr constant [14 x i8] c"stateAction2:\00", align 1
@state_action_2_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@sel_name.773 = private unnamed_addr constant [14 x i8] c"stateAction3:\00", align 1
@state_action_3_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@text_content.774 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@class_name.775 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.776 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.777 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.778 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.779 = private unnamed_addr constant [10 x i8] c"boolValue\00", align 1
@sel_name.780 = private unnamed_addr constant [23 x i8] c"removeArrangedSubview:\00", align 1
@sel_name.781 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.782 = private unnamed_addr constant [20 x i8] c"removeFromSuperview\00", align 1
@sel_name.783 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.784 = private unnamed_addr constant [31 x i8] c"insertArrangedSubview:atIndex:\00", align 1
@sel_name.785 = private unnamed_addr constant [15 x i8] c"layoutIfNeeded\00", align 1
@sel_name.786 = private unnamed_addr constant [14 x i8] c"stateAction3:\00", align 1
@sel_name.787 = private unnamed_addr constant [14 x i8] c"stateAction2:\00", align 1
@sel_name.788 = private unnamed_addr constant [14 x i8] c"stateAction4:\00", align 1
@state_action_4_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@sel_name.789 = private unnamed_addr constant [5 x i8] c"text\00", align 1
@sel_name.790 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.791 = private unnamed_addr constant [10 x i8] c"boolValue\00", align 1
@sel_name.792 = private unnamed_addr constant [23 x i8] c"removeArrangedSubview:\00", align 1
@sel_name.793 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.794 = private unnamed_addr constant [20 x i8] c"removeFromSuperview\00", align 1
@sel_name.795 = private unnamed_addr constant [11 x i8] c"setHidden:\00", align 1
@sel_name.796 = private unnamed_addr constant [31 x i8] c"insertArrangedSubview:atIndex:\00", align 1
@sel_name.797 = private unnamed_addr constant [15 x i8] c"layoutIfNeeded\00", align 1
@sel_name.798 = private unnamed_addr constant [14 x i8] c"stateAction3:\00", align 1
@sel_name.799 = private unnamed_addr constant [14 x i8] c"stateAction2:\00", align 1
@sel_name.800 = private unnamed_addr constant [22 x i8] c"updateStateFromInput:\00", align 1
@input_callback_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@sel_name.801 = private unnamed_addr constant [14 x i8] c"contentOffset\00", align 1
@sel_name.802 = private unnamed_addr constant [21 x i8] c"adjustedContentInset\00", align 1
@sel_name.803 = private unnamed_addr constant [10 x i8] c"setAlpha:\00", align 1
@sel_name.804 = private unnamed_addr constant [10 x i8] c"setAlpha:\00", align 1
@sel_name.805 = private unnamed_addr constant [21 x i8] c"scrollViewDidScroll:\00", align 1
@scroll_fade_callback_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@sel_name.806 = private unnamed_addr constant [12 x i8] c"focusEvent:\00", align 1
@focus_callback_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@text_content.807 = private unnamed_addr constant [12 x i8] c"AppDelegate\00", align 1
@class_name.808 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.809 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.810 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1

declare ptr @objc_getClass(ptr)

declare ptr @sel_registerName(ptr)

declare ptr @objc_msgSend(ptr, ptr, ...)

define i32 @main(i32 %0, ptr %1) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name)
  %app_delegate_class = call ptr @objc_allocateClassPair(ptr %cls, ptr @app_delegate_class_name, i64 0)
  %sel = call ptr @sel_registerName(ptr @sel_name.741)
  %2 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel, ptr @app_delegate_did_finish_launching, ptr @method_type_encoding)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.756)
  %3 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel1, ptr @app_delegate_state_action_0, ptr @state_action_0_encoding)
  %sel2 = call ptr @sel_registerName(ptr @sel_name.771)
  %4 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel2, ptr @app_delegate_state_action_1, ptr @state_action_1_encoding)
  %sel3 = call ptr @sel_registerName(ptr @sel_name.772)
  %5 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel3, ptr @app_delegate_state_action_2, ptr @state_action_2_encoding)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.773)
  %6 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel4, ptr @app_delegate_state_action_3, ptr @state_action_3_encoding)
  %sel5 = call ptr @sel_registerName(ptr @sel_name.788)
  %7 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel5, ptr @app_delegate_state_action_4, ptr @state_action_4_encoding)
  %sel6 = call ptr @sel_registerName(ptr @sel_name.800)
  %8 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel6, ptr @app_delegate_update_state_from_input, ptr @input_callback_encoding)
  %sel7 = call ptr @sel_registerName(ptr @sel_name.805)
  %9 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel7, ptr @app_delegate_scroll_view_did_scroll, ptr @scroll_fade_callback_encoding)
  %sel8 = call ptr @sel_registerName(ptr @sel_name.806)
  %10 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel8, ptr @app_delegate_focus_event, ptr @focus_callback_encoding)
  call void @objc_registerClassPair(ptr %app_delegate_class)
  %cls9 = call ptr @objc_getClass(ptr @class_name.808)
  %sel10 = call ptr @sel_registerName(ptr @sel_name.809)
  %msg_result = call ptr @objc_msgSend(ptr %cls9, ptr %sel10)
  %sel11 = call ptr @sel_registerName(ptr @sel_name.810)
  %msg_result12 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel11, ptr @text_content.807)
  %11 = call i32 @UIApplicationMain(i32 %0, ptr %1, ptr null, ptr %msg_result12)
  ret i32 0
}

declare ptr @objc_allocateClassPair(ptr, ptr, i64)

declare i1 @class_addMethod(ptr, ptr, ptr, ptr)

declare void @objc_registerClassPair(ptr)

define i8 @app_delegate_did_finish_launching(ptr %0, ptr %1, ptr %2, ptr %3) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name.1)
  %sel = call ptr @sel_registerName(ptr @sel_name)
  %msg_result = call ptr @objc_msgSend(ptr %cls, ptr %sel)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.2)
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, ptr @text_content)
  store ptr %msg_result2, ptr @g_state_value, align 8
  %cls3 = call ptr @objc_getClass(ptr @class_name.3)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.4)
  %msg_result5 = call ptr @objc_msgSend(ptr %cls3, ptr %sel4)
  %sel6 = call ptr @sel_registerName(ptr @sel_name.5)
  %msg_result7 = call ptr @objc_msgSend(ptr %msg_result5, ptr %sel6, { { double, double }, { double, double } } { { double, double } zeroinitializer, { double, double } { double 3.500000e+02, double 7.000000e+02 } })
  %cls8 = call ptr @objc_getClass(ptr @class_name.6)
  %sel9 = call ptr @sel_registerName(ptr @sel_name.7)
  %msg_result10 = call ptr @objc_msgSend(ptr %cls8, ptr %sel9, double 0x3FEE5E5E5E5E5E5E, double 0x3FEEBEBEBEBEBEBF, double 0x3FEF5F5F5F5F5F5F, double 1.000000e+00)
  %sel11 = call ptr @sel_registerName(ptr @sel_name.8)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel11, ptr %msg_result10)
  %sel12 = call ptr @sel_registerName(ptr @sel_name.9)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel12, i64 1)
  %sel13 = call ptr @sel_registerName(ptr @sel_name.10)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel13, double 0.000000e+00)
  %sel14 = call ptr @sel_registerName(ptr @sel_name.11)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel14, i64 0)
  %sel15 = call ptr @sel_registerName(ptr @sel_name.12)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel15, i64 0)
  %sel16 = call ptr @sel_registerName(ptr @sel_name.13)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel16, i8 1)
  %sel17 = call ptr @sel_registerName(ptr @sel_name.14)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel17, { double, double, double, double } zeroinitializer)
  %cls18 = call ptr @objc_getClass(ptr @class_name.15)
  %sel19 = call ptr @sel_registerName(ptr @sel_name.16)
  %msg_result20 = call ptr @objc_msgSend(ptr %cls18, ptr %sel19)
  %sel21 = call ptr @sel_registerName(ptr @sel_name.17)
  %msg_result22 = call ptr @objc_msgSend(ptr %msg_result20, ptr %sel21, { { double, double }, { double, double } } { { double, double } zeroinitializer, { double, double } { double 3.500000e+02, double 4.400000e+01 } })
  %cls23 = call ptr @objc_getClass(ptr @class_name.18)
  %sel24 = call ptr @sel_registerName(ptr @sel_name.19)
  %msg_result25 = call ptr @objc_msgSend(ptr %cls23, ptr %sel24, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel26 = call ptr @sel_registerName(ptr @sel_name.20)
  call void @objc_msgSend(ptr %msg_result22, ptr %sel26, ptr %msg_result25)
  store ptr %msg_result22, ptr @g_top_background_view, align 8
  store ptr %msg_result22, ptr @g_scroll_fade_view, align 8
  %sel27 = call ptr @sel_registerName(ptr @sel_name.21)
  call void @objc_msgSend(ptr %msg_result22, ptr %sel27, double 0.000000e+00)
  %cls28 = call ptr @objc_getClass(ptr @class_name.22)
  %sel29 = call ptr @sel_registerName(ptr @sel_name.23)
  %msg_result30 = call ptr @objc_msgSend(ptr %cls28, ptr %sel29)
  %sel31 = call ptr @sel_registerName(ptr @sel_name.24)
  %msg_result32 = call ptr @objc_msgSend(ptr %msg_result30, ptr %sel31, { { double, double }, { double, double } } { { double, double } { double 1.200000e+01, double 1.200000e+01 }, { double, double } { double 1.584000e+02, double 2.000000e+01 } })
  %cls33 = call ptr @objc_getClass(ptr @class_name.26)
  %sel34 = call ptr @sel_registerName(ptr @sel_name.27)
  %msg_result35 = call ptr @objc_msgSend(ptr %cls33, ptr %sel34)
  %sel36 = call ptr @sel_registerName(ptr @sel_name.28)
  %msg_result37 = call ptr @objc_msgSend(ptr %msg_result35, ptr %sel36, ptr @text_content.25)
  %sel38 = call ptr @sel_registerName(ptr @sel_name.29)
  call void @objc_msgSend(ptr %msg_result32, ptr %sel38, ptr %msg_result37)
  %sel39 = call ptr @sel_registerName(ptr @sel_name.30)
  call void @objc_msgSend(ptr %msg_result32, ptr %sel39, i64 1)
  %cls40 = call ptr @objc_getClass(ptr @class_name.31)
  %sel41 = call ptr @sel_registerName(ptr @sel_name.32)
  %msg_result42 = call ptr @objc_msgSend(ptr %cls40, ptr %sel41)
  %sel43 = call ptr @sel_registerName(ptr @sel_name.33)
  call void @objc_msgSend(ptr %msg_result32, ptr %sel43, ptr %msg_result42)
  %cls44 = call ptr @objc_getClass(ptr @class_name.34)
  %sel45 = call ptr @sel_registerName(ptr @sel_name.35)
  %msg_result46 = call ptr @objc_msgSend(ptr %cls44, ptr %sel45, double 2.200000e+01)
  %sel47 = call ptr @sel_registerName(ptr @sel_name.36)
  call void @objc_msgSend(ptr %msg_result32, ptr %sel47, ptr %msg_result46)
  %sel48 = call ptr @sel_registerName(ptr @sel_name.37)
  call void @objc_msgSend(ptr %msg_result22, ptr %sel48, ptr %msg_result32)
  %cls49 = call ptr @objc_getClass(ptr @class_name.38)
  %sel50 = call ptr @sel_registerName(ptr @sel_name.39)
  %msg_result51 = call ptr @objc_msgSend(ptr %cls49, ptr %sel50)
  %sel52 = call ptr @sel_registerName(ptr @sel_name.40)
  %msg_result53 = call ptr @objc_msgSend(ptr %msg_result51, ptr %sel52, { { double, double }, { double, double } } { { double, double } { double 2.960000e+02, double 1.360000e+01 }, { double, double } { double 4.200000e+01, double 1.680000e+01 } })
  %cls54 = call ptr @objc_getClass(ptr @class_name.42)
  %sel55 = call ptr @sel_registerName(ptr @sel_name.43)
  %msg_result56 = call ptr @objc_msgSend(ptr %cls54, ptr %sel55)
  %sel57 = call ptr @sel_registerName(ptr @sel_name.44)
  %msg_result58 = call ptr @objc_msgSend(ptr %msg_result56, ptr %sel57, ptr @text_content.41)
  %sel59 = call ptr @sel_registerName(ptr @sel_name.45)
  call void @objc_msgSend(ptr %msg_result53, ptr %sel59, ptr %msg_result58)
  %sel60 = call ptr @sel_registerName(ptr @sel_name.46)
  call void @objc_msgSend(ptr %msg_result53, ptr %sel60, i64 1)
  %cls61 = call ptr @objc_getClass(ptr @class_name.47)
  %sel62 = call ptr @sel_registerName(ptr @sel_name.48)
  %msg_result63 = call ptr @objc_msgSend(ptr %cls61, ptr %sel62, double 0x3FE7D7D7D7D7D7D8, double 0x3FE9B9B9B9B9B9BA, double 0x3FECDCDCDCDCDCDD, double 1.000000e+00)
  %sel64 = call ptr @sel_registerName(ptr @sel_name.49)
  call void @objc_msgSend(ptr %msg_result53, ptr %sel64, ptr %msg_result63)
  %cls65 = call ptr @objc_getClass(ptr @class_name.50)
  %sel66 = call ptr @sel_registerName(ptr @sel_name.51)
  %msg_result67 = call ptr @objc_msgSend(ptr %cls65, ptr %sel66, double 1.400000e+01)
  %sel68 = call ptr @sel_registerName(ptr @sel_name.52)
  call void @objc_msgSend(ptr %msg_result53, ptr %sel68, ptr %msg_result67)
  %sel69 = call ptr @sel_registerName(ptr @sel_name.53)
  call void @objc_msgSend(ptr %msg_result22, ptr %sel69, ptr %msg_result53)
  %sel70 = call ptr @sel_registerName(ptr @sel_name.54)
  call void @objc_msgSend(ptr %msg_result32, ptr %sel70, i1 false)
  %sel71 = call ptr @sel_registerName(ptr @sel_name.55)
  %msg_result72 = call ptr @objc_msgSend(ptr %msg_result32, ptr %sel71)
  %sel73 = call ptr @sel_registerName(ptr @sel_name.56)
  %msg_result74 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel73)
  %sel75 = call ptr @sel_registerName(ptr @sel_name.57)
  %msg_result76 = call ptr @objc_msgSend(ptr %msg_result72, ptr %sel75, ptr %msg_result74, double 0.000000e+00)
  %sel77 = call ptr @sel_registerName(ptr @sel_name.58)
  call void @objc_msgSend(ptr %msg_result76, ptr %sel77, i1 true)
  %sel78 = call ptr @sel_registerName(ptr @sel_name.59)
  %msg_result79 = call ptr @objc_msgSend(ptr %msg_result32, ptr %sel78)
  %sel80 = call ptr @sel_registerName(ptr @sel_name.60)
  %msg_result81 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel80)
  %sel82 = call ptr @sel_registerName(ptr @sel_name.61)
  %msg_result83 = call ptr @objc_msgSend(ptr %msg_result79, ptr %sel82, ptr %msg_result81, double 1.200000e+01)
  %sel84 = call ptr @sel_registerName(ptr @sel_name.62)
  call void @objc_msgSend(ptr %msg_result83, ptr %sel84, i1 true)
  %sel85 = call ptr @sel_registerName(ptr @sel_name.63)
  call void @objc_msgSend(ptr %msg_result53, ptr %sel85, i1 false)
  %sel86 = call ptr @sel_registerName(ptr @sel_name.64)
  %msg_result87 = call ptr @objc_msgSend(ptr %msg_result53, ptr %sel86)
  %sel88 = call ptr @sel_registerName(ptr @sel_name.65)
  %msg_result89 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel88)
  %sel90 = call ptr @sel_registerName(ptr @sel_name.66)
  %msg_result91 = call ptr @objc_msgSend(ptr %msg_result87, ptr %sel90, ptr %msg_result89, double 0.000000e+00)
  %sel92 = call ptr @sel_registerName(ptr @sel_name.67)
  call void @objc_msgSend(ptr %msg_result91, ptr %sel92, i1 true)
  %sel93 = call ptr @sel_registerName(ptr @sel_name.68)
  %msg_result94 = call ptr @objc_msgSend(ptr %msg_result53, ptr %sel93)
  %sel95 = call ptr @sel_registerName(ptr @sel_name.69)
  %msg_result96 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel95)
  %sel97 = call ptr @sel_registerName(ptr @sel_name.70)
  %msg_result98 = call ptr @objc_msgSend(ptr %msg_result94, ptr %sel97, ptr %msg_result96, double -1.200000e+01)
  %sel99 = call ptr @sel_registerName(ptr @sel_name.71)
  call void @objc_msgSend(ptr %msg_result98, ptr %sel99, i1 true)
  %sel100 = call ptr @sel_registerName(ptr @sel_name.72)
  call void @objc_msgSend(ptr %msg_result22, ptr %sel100, i8 0)
  %sel101 = call ptr @sel_registerName(ptr @sel_name.73)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel101, ptr %msg_result22)
  %sel102 = call ptr @sel_registerName(ptr @sel_name.74)
  %msg_result103 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel102)
  %sel104 = call ptr @sel_registerName(ptr @sel_name.75)
  %msg_result105 = call ptr @objc_msgSend(ptr %msg_result7, ptr %sel104)
  %sel106 = call ptr @sel_registerName(ptr @sel_name.76)
  %msg_result107 = call ptr @objc_msgSend(ptr %msg_result103, ptr %sel106, ptr %msg_result105, double 1.000000e+00, double -0.000000e+00)
  %sel108 = call ptr @sel_registerName(ptr @sel_name.77)
  call void @objc_msgSend(ptr %msg_result107, ptr %sel108, i1 true)
  %sel109 = call ptr @sel_registerName(ptr @sel_name.78)
  %msg_result110 = call ptr @objc_msgSend(ptr %msg_result22, ptr %sel109)
  %sel111 = call ptr @sel_registerName(ptr @sel_name.79)
  %msg_result112 = call ptr @objc_msgSend(ptr %msg_result110, ptr %sel111, double 4.400000e+01)
  %sel113 = call ptr @sel_registerName(ptr @sel_name.80)
  call void @objc_msgSend(ptr %msg_result112, ptr %sel113, i1 true)
  %cls114 = call ptr @objc_getClass(ptr @class_name.81)
  %sel115 = call ptr @sel_registerName(ptr @sel_name.82)
  %msg_result116 = call ptr @objc_msgSend(ptr %cls114, ptr %sel115)
  %sel117 = call ptr @sel_registerName(ptr @sel_name.83)
  %msg_result118 = call ptr @objc_msgSend(ptr %msg_result116, ptr %sel117, { { double, double }, { double, double } } { { double, double } { double 0.000000e+00, double 4.400000e+01 }, { double, double } { double 3.500000e+02, double 5.640000e+02 } })
  %cls119 = call ptr @objc_getClass(ptr @class_name.84)
  %sel120 = call ptr @sel_registerName(ptr @sel_name.85)
  %msg_result121 = call ptr @objc_msgSend(ptr %cls119, ptr %sel120)
  %sel122 = call ptr @sel_registerName(ptr @sel_name.86)
  call void @objc_msgSend(ptr %msg_result118, ptr %sel122, ptr %msg_result121)
  %sel123 = call ptr @sel_registerName(ptr @sel_name.87)
  %msg_result124 = call ptr @objc_msgSend(ptr %msg_result118, ptr %sel123)
  %sel125 = call ptr @sel_registerName(ptr @sel_name.88)
  call void @objc_msgSend(ptr %msg_result124, ptr %sel125, double 1.800000e+01)
  %sel126 = call ptr @sel_registerName(ptr @sel_name.89)
  call void @objc_msgSend(ptr %msg_result118, ptr %sel126, ptr %0)
  store ptr %msg_result118, ptr @g_scroll_fade_scroll, align 8
  %cls127 = call ptr @objc_getClass(ptr @class_name.90)
  %sel128 = call ptr @sel_registerName(ptr @sel_name.91)
  %msg_result129 = call ptr @objc_msgSend(ptr %cls127, ptr %sel128)
  %sel130 = call ptr @sel_registerName(ptr @sel_name.92)
  %msg_result131 = call ptr @objc_msgSend(ptr %msg_result129, ptr %sel130, { { double, double }, { double, double } } { { double, double } zeroinitializer, { double, double } { double 3.500000e+02, double 5.640000e+02 } })
  %sel132 = call ptr @sel_registerName(ptr @sel_name.93)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel132, i64 1)
  %sel133 = call ptr @sel_registerName(ptr @sel_name.94)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel133, double 1.400000e+01)
  %sel134 = call ptr @sel_registerName(ptr @sel_name.95)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel134, i64 0)
  %sel135 = call ptr @sel_registerName(ptr @sel_name.96)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel135, i64 0)
  %sel136 = call ptr @sel_registerName(ptr @sel_name.97)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel136, i8 1)
  %sel137 = call ptr @sel_registerName(ptr @sel_name.98)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel137, { double, double, double, double } { double 1.600000e+01, double 1.600000e+01, double 1.600000e+01, double 1.600000e+01 })
  %sel138 = call ptr @sel_registerName(ptr @sel_name.99)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel138, i8 0)
  %sel139 = call ptr @sel_registerName(ptr @sel_name.100)
  call void @objc_msgSend(ptr %msg_result118, ptr %sel139, ptr %msg_result131)
  %sel140 = call ptr @sel_registerName(ptr @sel_name.101)
  %msg_result141 = call ptr @objc_msgSend(ptr %msg_result118, ptr %sel140)
  %sel142 = call ptr @sel_registerName(ptr @sel_name.102)
  %msg_result143 = call ptr @objc_msgSend(ptr %msg_result118, ptr %sel142)
  %sel144 = call ptr @sel_registerName(ptr @sel_name.103)
  %msg_result145 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel144)
  %sel146 = call ptr @sel_registerName(ptr @sel_name.104)
  %msg_result147 = call ptr @objc_msgSend(ptr %msg_result141, ptr %sel146)
  %sel148 = call ptr @sel_registerName(ptr @sel_name.105)
  %msg_result149 = call ptr @objc_msgSend(ptr %msg_result145, ptr %sel148, ptr %msg_result147, double 0.000000e+00)
  %sel150 = call ptr @sel_registerName(ptr @sel_name.106)
  call void @objc_msgSend(ptr %msg_result149, ptr %sel150, i1 true)
  %sel151 = call ptr @sel_registerName(ptr @sel_name.107)
  %msg_result152 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel151)
  %sel153 = call ptr @sel_registerName(ptr @sel_name.108)
  %msg_result154 = call ptr @objc_msgSend(ptr %msg_result141, ptr %sel153)
  %sel155 = call ptr @sel_registerName(ptr @sel_name.109)
  %msg_result156 = call ptr @objc_msgSend(ptr %msg_result152, ptr %sel155, ptr %msg_result154, double 0.000000e+00)
  %sel157 = call ptr @sel_registerName(ptr @sel_name.110)
  call void @objc_msgSend(ptr %msg_result156, ptr %sel157, i1 true)
  %sel158 = call ptr @sel_registerName(ptr @sel_name.111)
  %msg_result159 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel158)
  %sel160 = call ptr @sel_registerName(ptr @sel_name.112)
  %msg_result161 = call ptr @objc_msgSend(ptr %msg_result141, ptr %sel160)
  %sel162 = call ptr @sel_registerName(ptr @sel_name.113)
  %msg_result163 = call ptr @objc_msgSend(ptr %msg_result159, ptr %sel162, ptr %msg_result161, double 0.000000e+00)
  %sel164 = call ptr @sel_registerName(ptr @sel_name.114)
  call void @objc_msgSend(ptr %msg_result163, ptr %sel164, i1 true)
  %sel165 = call ptr @sel_registerName(ptr @sel_name.115)
  %msg_result166 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel165)
  %sel167 = call ptr @sel_registerName(ptr @sel_name.116)
  %msg_result168 = call ptr @objc_msgSend(ptr %msg_result141, ptr %sel167)
  %sel169 = call ptr @sel_registerName(ptr @sel_name.117)
  %msg_result170 = call ptr @objc_msgSend(ptr %msg_result166, ptr %sel169, ptr %msg_result168, double 0.000000e+00)
  %sel171 = call ptr @sel_registerName(ptr @sel_name.118)
  call void @objc_msgSend(ptr %msg_result170, ptr %sel171, i1 true)
  %sel172 = call ptr @sel_registerName(ptr @sel_name.119)
  %msg_result173 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel172)
  %sel174 = call ptr @sel_registerName(ptr @sel_name.120)
  %msg_result175 = call ptr @objc_msgSend(ptr %msg_result143, ptr %sel174)
  %sel176 = call ptr @sel_registerName(ptr @sel_name.121)
  %msg_result177 = call ptr @objc_msgSend(ptr %msg_result173, ptr %sel176, ptr %msg_result175, double 0.000000e+00)
  %sel178 = call ptr @sel_registerName(ptr @sel_name.122)
  call void @objc_msgSend(ptr %msg_result177, ptr %sel178, i1 true)
  %sel179 = call ptr @sel_registerName(ptr @sel_name.123)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel179, i64 1)
  %sel180 = call ptr @sel_registerName(ptr @sel_name.124)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel180, double 1.400000e+01)
  %sel181 = call ptr @sel_registerName(ptr @sel_name.125)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel181, i64 0)
  %sel182 = call ptr @sel_registerName(ptr @sel_name.126)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel182, i64 0)
  %sel183 = call ptr @sel_registerName(ptr @sel_name.127)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel183, i8 1)
  %sel184 = call ptr @sel_registerName(ptr @sel_name.128)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel184, { double, double, double, double } { double 1.600000e+01, double 1.600000e+01, double 1.600000e+01, double 1.600000e+01 })
  %cls185 = call ptr @objc_getClass(ptr @class_name.129)
  %sel186 = call ptr @sel_registerName(ptr @sel_name.130)
  %msg_result187 = call ptr @objc_msgSend(ptr %cls185, ptr %sel186)
  %sel188 = call ptr @sel_registerName(ptr @sel_name.131)
  %msg_result189 = call ptr @objc_msgSend(ptr %msg_result187, ptr %sel188, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 1.600000e+01 }, { double, double } { double 3.180000e+02, double 3.120000e+01 } })
  %cls190 = call ptr @objc_getClass(ptr @class_name.133)
  %sel191 = call ptr @sel_registerName(ptr @sel_name.134)
  %msg_result192 = call ptr @objc_msgSend(ptr %cls190, ptr %sel191)
  %sel193 = call ptr @sel_registerName(ptr @sel_name.135)
  %msg_result194 = call ptr @objc_msgSend(ptr %msg_result192, ptr %sel193, ptr @text_content.132)
  %sel195 = call ptr @sel_registerName(ptr @sel_name.136)
  call void @objc_msgSend(ptr %msg_result189, ptr %sel195, ptr %msg_result194)
  %cls196 = call ptr @objc_getClass(ptr @class_name.137)
  %sel197 = call ptr @sel_registerName(ptr @sel_name.138)
  %msg_result198 = call ptr @objc_msgSend(ptr %cls196, ptr %sel197, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel199 = call ptr @sel_registerName(ptr @sel_name.139)
  call void @objc_msgSend(ptr %msg_result189, ptr %sel199, ptr %msg_result198)
  %cls200 = call ptr @objc_getClass(ptr @class_name.140)
  %sel201 = call ptr @sel_registerName(ptr @sel_name.141)
  %msg_result202 = call ptr @objc_msgSend(ptr %cls200, ptr %sel201, double 2.600000e+01)
  %sel203 = call ptr @sel_registerName(ptr @sel_name.142)
  call void @objc_msgSend(ptr %msg_result189, ptr %sel203, ptr %msg_result202)
  %sel204 = call ptr @sel_registerName(ptr @sel_name.143)
  call void @objc_msgSend(ptr %msg_result189, ptr %sel204, i8 0)
  %sel205 = call ptr @sel_registerName(ptr @sel_name.144)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel205, ptr %msg_result189)
  %cls206 = call ptr @objc_getClass(ptr @class_name.145)
  %sel207 = call ptr @sel_registerName(ptr @sel_name.146)
  %msg_result208 = call ptr @objc_msgSend(ptr %cls206, ptr %sel207)
  %sel209 = call ptr @sel_registerName(ptr @sel_name.147)
  %msg_result210 = call ptr @objc_msgSend(ptr %msg_result208, ptr %sel209, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 6.120000e+01 }, { double, double } { double 3.180000e+02, double 1.920000e+01 } })
  %cls211 = call ptr @objc_getClass(ptr @class_name.149)
  %sel212 = call ptr @sel_registerName(ptr @sel_name.150)
  %msg_result213 = call ptr @objc_msgSend(ptr %cls211, ptr %sel212)
  %sel214 = call ptr @sel_registerName(ptr @sel_name.151)
  %msg_result215 = call ptr @objc_msgSend(ptr %msg_result213, ptr %sel214, ptr @text_content.148)
  %sel216 = call ptr @sel_registerName(ptr @sel_name.152)
  call void @objc_msgSend(ptr %msg_result210, ptr %sel216, ptr %msg_result215)
  %cls217 = call ptr @objc_getClass(ptr @class_name.153)
  %sel218 = call ptr @sel_registerName(ptr @sel_name.154)
  %msg_result219 = call ptr @objc_msgSend(ptr %cls217, ptr %sel218)
  %sel220 = call ptr @sel_registerName(ptr @sel_name.155)
  call void @objc_msgSend(ptr %msg_result210, ptr %sel220, ptr %msg_result219)
  %cls221 = call ptr @objc_getClass(ptr @class_name.156)
  %sel222 = call ptr @sel_registerName(ptr @sel_name.157)
  %msg_result223 = call ptr @objc_msgSend(ptr %cls221, ptr %sel222, double 1.600000e+01)
  %sel224 = call ptr @sel_registerName(ptr @sel_name.158)
  call void @objc_msgSend(ptr %msg_result210, ptr %sel224, ptr %msg_result223)
  %sel225 = call ptr @sel_registerName(ptr @sel_name.159)
  call void @objc_msgSend(ptr %msg_result210, ptr %sel225, i8 0)
  %sel226 = call ptr @sel_registerName(ptr @sel_name.160)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel226, ptr %msg_result210)
  %cls227 = call ptr @objc_getClass(ptr @class_name.161)
  %sel228 = call ptr @sel_registerName(ptr @sel_name.162)
  %msg_result229 = call ptr @objc_msgSend(ptr %cls227, ptr %sel228)
  %sel230 = call ptr @sel_registerName(ptr @sel_name.163)
  %msg_result231 = call ptr @objc_msgSend(ptr %msg_result229, ptr %sel230, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 9.440000e+01 }, { double, double } { double 3.180000e+02, double 1.200000e+02 } })
  %cls232 = call ptr @objc_getClass(ptr @class_name.164)
  %sel233 = call ptr @sel_registerName(ptr @sel_name.165)
  %msg_result234 = call ptr @objc_msgSend(ptr %cls232, ptr %sel233, double 0x3FD1515151515151, double 0x3FDA1A1A1A1A1A1A, double 0x3FEB9B9B9B9B9B9C, double 1.000000e+00)
  %sel235 = call ptr @sel_registerName(ptr @sel_name.166)
  call void @objc_msgSend(ptr %msg_result231, ptr %sel235, ptr %msg_result234)
  %sel236 = call ptr @sel_registerName(ptr @sel_name.167)
  %msg_result237 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel236)
  %sel238 = call ptr @sel_registerName(ptr @sel_name.168)
  call void @objc_msgSend(ptr %msg_result237, ptr %sel238, double 1.600000e+01)
  %cls239 = call ptr @objc_getClass(ptr @class_name.169)
  %sel240 = call ptr @sel_registerName(ptr @sel_name.170)
  %msg_result241 = call ptr @objc_msgSend(ptr %cls239, ptr %sel240)
  %sel242 = call ptr @sel_registerName(ptr @sel_name.171)
  %msg_result243 = call ptr @objc_msgSend(ptr %msg_result241, ptr %sel242, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.800000e+01 }, { double, double } { double 2.820000e+02, double 1.680000e+01 } })
  %cls244 = call ptr @objc_getClass(ptr @class_name.173)
  %sel245 = call ptr @sel_registerName(ptr @sel_name.174)
  %msg_result246 = call ptr @objc_msgSend(ptr %cls244, ptr %sel245)
  %sel247 = call ptr @sel_registerName(ptr @sel_name.175)
  %msg_result248 = call ptr @objc_msgSend(ptr %msg_result246, ptr %sel247, ptr @text_content.172)
  %sel249 = call ptr @sel_registerName(ptr @sel_name.176)
  call void @objc_msgSend(ptr %msg_result243, ptr %sel249, ptr %msg_result248)
  %cls250 = call ptr @objc_getClass(ptr @class_name.177)
  %sel251 = call ptr @sel_registerName(ptr @sel_name.178)
  %msg_result252 = call ptr @objc_msgSend(ptr %cls250, ptr %sel251, double 0x3FEB9B9B9B9B9B9C, double 0x3FECDCDCDCDCDCDD, double 1.000000e+00, double 1.000000e+00)
  %sel253 = call ptr @sel_registerName(ptr @sel_name.179)
  call void @objc_msgSend(ptr %msg_result243, ptr %sel253, ptr %msg_result252)
  %cls254 = call ptr @objc_getClass(ptr @class_name.180)
  %sel255 = call ptr @sel_registerName(ptr @sel_name.181)
  %msg_result256 = call ptr @objc_msgSend(ptr %cls254, ptr %sel255, double 1.400000e+01)
  %sel257 = call ptr @sel_registerName(ptr @sel_name.182)
  call void @objc_msgSend(ptr %msg_result243, ptr %sel257, ptr %msg_result256)
  %sel258 = call ptr @sel_registerName(ptr @sel_name.183)
  call void @objc_msgSend(ptr %msg_result231, ptr %sel258, ptr %msg_result243)
  %cls259 = call ptr @objc_getClass(ptr @class_name.184)
  %sel260 = call ptr @sel_registerName(ptr @sel_name.185)
  %msg_result261 = call ptr @objc_msgSend(ptr %cls259, ptr %sel260)
  %sel262 = call ptr @sel_registerName(ptr @sel_name.186)
  %msg_result263 = call ptr @objc_msgSend(ptr %msg_result261, ptr %sel262, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 7.560000e+01 }, { double, double } { double 2.820000e+02, double 2.640000e+01 } })
  %cls264 = call ptr @objc_getClass(ptr @class_name.188)
  %sel265 = call ptr @sel_registerName(ptr @sel_name.189)
  %msg_result266 = call ptr @objc_msgSend(ptr %cls264, ptr %sel265)
  %sel267 = call ptr @sel_registerName(ptr @sel_name.190)
  %msg_result268 = call ptr @objc_msgSend(ptr %msg_result266, ptr %sel267, ptr @text_content.187)
  %sel269 = call ptr @sel_registerName(ptr @sel_name.191)
  call void @objc_msgSend(ptr %msg_result263, ptr %sel269, ptr %msg_result268)
  %cls270 = call ptr @objc_getClass(ptr @class_name.192)
  %sel271 = call ptr @sel_registerName(ptr @sel_name.193)
  %msg_result272 = call ptr @objc_msgSend(ptr %cls270, ptr %sel271)
  %sel273 = call ptr @sel_registerName(ptr @sel_name.194)
  call void @objc_msgSend(ptr %msg_result263, ptr %sel273, ptr %msg_result272)
  %cls274 = call ptr @objc_getClass(ptr @class_name.195)
  %sel275 = call ptr @sel_registerName(ptr @sel_name.196)
  %msg_result276 = call ptr @objc_msgSend(ptr %cls274, ptr %sel275, double 2.200000e+01)
  %sel277 = call ptr @sel_registerName(ptr @sel_name.197)
  call void @objc_msgSend(ptr %msg_result263, ptr %sel277, ptr %msg_result276)
  store ptr %msg_result263, ptr @g_state_label_0, align 8
  %sel278 = call ptr @sel_registerName(ptr @sel_name.198)
  call void @objc_msgSend(ptr %msg_result231, ptr %sel278, ptr %msg_result263)
  %sel279 = call ptr @sel_registerName(ptr @sel_name.199)
  call void @objc_msgSend(ptr %msg_result243, ptr %sel279, i1 false)
  %sel280 = call ptr @sel_registerName(ptr @sel_name.200)
  %msg_result281 = call ptr @objc_msgSend(ptr %msg_result243, ptr %sel280)
  %sel282 = call ptr @sel_registerName(ptr @sel_name.201)
  %msg_result283 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel282)
  %sel284 = call ptr @sel_registerName(ptr @sel_name.202)
  %msg_result285 = call ptr @objc_msgSend(ptr %msg_result281, ptr %sel284, ptr %msg_result283, double 1.800000e+01)
  %sel286 = call ptr @sel_registerName(ptr @sel_name.203)
  call void @objc_msgSend(ptr %msg_result285, ptr %sel286, i1 true)
  %sel287 = call ptr @sel_registerName(ptr @sel_name.204)
  %msg_result288 = call ptr @objc_msgSend(ptr %msg_result243, ptr %sel287)
  %sel289 = call ptr @sel_registerName(ptr @sel_name.205)
  %msg_result290 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel289)
  %sel291 = call ptr @sel_registerName(ptr @sel_name.206)
  %msg_result292 = call ptr @objc_msgSend(ptr %msg_result288, ptr %sel291, ptr %msg_result290, double -1.800000e+01)
  %sel293 = call ptr @sel_registerName(ptr @sel_name.207)
  call void @objc_msgSend(ptr %msg_result292, ptr %sel293, i1 true)
  %sel294 = call ptr @sel_registerName(ptr @sel_name.208)
  %msg_result295 = call ptr @objc_msgSend(ptr %msg_result243, ptr %sel294)
  %sel296 = call ptr @sel_registerName(ptr @sel_name.209)
  %msg_result297 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel296)
  %sel298 = call ptr @sel_registerName(ptr @sel_name.210)
  %msg_result299 = call ptr @objc_msgSend(ptr %msg_result295, ptr %sel298, ptr %msg_result297, double 1.800000e+01)
  %sel300 = call ptr @sel_registerName(ptr @sel_name.211)
  call void @objc_msgSend(ptr %msg_result299, ptr %sel300, i1 true)
  %sel301 = call ptr @sel_registerName(ptr @sel_name.212)
  call void @objc_msgSend(ptr %msg_result263, ptr %sel301, i1 false)
  %sel302 = call ptr @sel_registerName(ptr @sel_name.213)
  %msg_result303 = call ptr @objc_msgSend(ptr %msg_result263, ptr %sel302)
  %sel304 = call ptr @sel_registerName(ptr @sel_name.214)
  %msg_result305 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel304)
  %sel306 = call ptr @sel_registerName(ptr @sel_name.215)
  %msg_result307 = call ptr @objc_msgSend(ptr %msg_result303, ptr %sel306, ptr %msg_result305, double 1.800000e+01)
  %sel308 = call ptr @sel_registerName(ptr @sel_name.216)
  call void @objc_msgSend(ptr %msg_result307, ptr %sel308, i1 true)
  %sel309 = call ptr @sel_registerName(ptr @sel_name.217)
  %msg_result310 = call ptr @objc_msgSend(ptr %msg_result263, ptr %sel309)
  %sel311 = call ptr @sel_registerName(ptr @sel_name.218)
  %msg_result312 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel311)
  %sel313 = call ptr @sel_registerName(ptr @sel_name.219)
  %msg_result314 = call ptr @objc_msgSend(ptr %msg_result310, ptr %sel313, ptr %msg_result312, double -1.800000e+01)
  %sel315 = call ptr @sel_registerName(ptr @sel_name.220)
  call void @objc_msgSend(ptr %msg_result314, ptr %sel315, i1 true)
  %sel316 = call ptr @sel_registerName(ptr @sel_name.221)
  %msg_result317 = call ptr @objc_msgSend(ptr %msg_result263, ptr %sel316)
  %sel318 = call ptr @sel_registerName(ptr @sel_name.222)
  %msg_result319 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel318)
  %sel320 = call ptr @sel_registerName(ptr @sel_name.223)
  %msg_result321 = call ptr @objc_msgSend(ptr %msg_result317, ptr %sel320, ptr %msg_result319, double -1.800000e+01)
  %sel322 = call ptr @sel_registerName(ptr @sel_name.224)
  call void @objc_msgSend(ptr %msg_result321, ptr %sel322, i1 true)
  %sel323 = call ptr @sel_registerName(ptr @sel_name.225)
  call void @objc_msgSend(ptr %msg_result231, ptr %sel323, i8 0)
  %sel324 = call ptr @sel_registerName(ptr @sel_name.226)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel324, ptr %msg_result231)
  %sel325 = call ptr @sel_registerName(ptr @sel_name.227)
  %msg_result326 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel325)
  %sel327 = call ptr @sel_registerName(ptr @sel_name.228)
  %msg_result328 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel327)
  %sel329 = call ptr @sel_registerName(ptr @sel_name.229)
  %msg_result330 = call ptr @objc_msgSend(ptr %msg_result326, ptr %sel329, ptr %msg_result328, double 1.000000e+00, double -3.200000e+01)
  %sel331 = call ptr @sel_registerName(ptr @sel_name.230)
  call void @objc_msgSend(ptr %msg_result330, ptr %sel331, i1 true)
  %sel332 = call ptr @sel_registerName(ptr @sel_name.231)
  %msg_result333 = call ptr @objc_msgSend(ptr %msg_result231, ptr %sel332)
  %sel334 = call ptr @sel_registerName(ptr @sel_name.232)
  %msg_result335 = call ptr @objc_msgSend(ptr %msg_result333, ptr %sel334, double 1.200000e+02)
  %sel336 = call ptr @sel_registerName(ptr @sel_name.233)
  call void @objc_msgSend(ptr %msg_result335, ptr %sel336, i1 true)
  %cls337 = call ptr @objc_getClass(ptr @class_name.234)
  %sel338 = call ptr @sel_registerName(ptr @sel_name.235)
  %msg_result339 = call ptr @objc_msgSend(ptr %cls337, ptr %sel338)
  %sel340 = call ptr @sel_registerName(ptr @sel_name.236)
  %msg_result341 = call ptr @objc_msgSend(ptr %msg_result339, ptr %sel340, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 2.284000e+02 }, { double, double } { double 3.180000e+02, double 0x4035999999999999 } })
  %cls342 = call ptr @objc_getClass(ptr @class_name.238)
  %sel343 = call ptr @sel_registerName(ptr @sel_name.239)
  %msg_result344 = call ptr @objc_msgSend(ptr %cls342, ptr %sel343)
  %sel345 = call ptr @sel_registerName(ptr @sel_name.240)
  %msg_result346 = call ptr @objc_msgSend(ptr %msg_result344, ptr %sel345, ptr @text_content.237)
  %sel347 = call ptr @sel_registerName(ptr @sel_name.241)
  call void @objc_msgSend(ptr %msg_result341, ptr %sel347, ptr %msg_result346)
  %cls348 = call ptr @objc_getClass(ptr @class_name.242)
  %sel349 = call ptr @sel_registerName(ptr @sel_name.243)
  %msg_result350 = call ptr @objc_msgSend(ptr %cls348, ptr %sel349, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel351 = call ptr @sel_registerName(ptr @sel_name.244)
  call void @objc_msgSend(ptr %msg_result341, ptr %sel351, ptr %msg_result350)
  %cls352 = call ptr @objc_getClass(ptr @class_name.245)
  %sel353 = call ptr @sel_registerName(ptr @sel_name.246)
  %msg_result354 = call ptr @objc_msgSend(ptr %cls352, ptr %sel353, double 1.800000e+01)
  %sel355 = call ptr @sel_registerName(ptr @sel_name.247)
  call void @objc_msgSend(ptr %msg_result341, ptr %sel355, ptr %msg_result354)
  %sel356 = call ptr @sel_registerName(ptr @sel_name.248)
  call void @objc_msgSend(ptr %msg_result341, ptr %sel356, i8 0)
  %sel357 = call ptr @sel_registerName(ptr @sel_name.249)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel357, ptr %msg_result341)
  %cls358 = call ptr @objc_getClass(ptr @class_name.250)
  %sel359 = call ptr @sel_registerName(ptr @sel_name.251)
  %msg_result360 = call ptr @objc_msgSend(ptr %cls358, ptr %sel359)
  %sel361 = call ptr @sel_registerName(ptr @sel_name.252)
  %msg_result362 = call ptr @objc_msgSend(ptr %msg_result360, ptr %sel361, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 2.640000e+02 }, { double, double } { double 3.180000e+02, double 5.400000e+01 } })
  %cls363 = call ptr @objc_getClass(ptr @class_name.253)
  %sel364 = call ptr @sel_registerName(ptr @sel_name.254)
  %msg_result365 = call ptr @objc_msgSend(ptr %cls363, ptr %sel364)
  %sel366 = call ptr @sel_registerName(ptr @sel_name.255)
  call void @objc_msgSend(ptr %msg_result362, ptr %sel366, ptr %msg_result365)
  %cls367 = call ptr @objc_getClass(ptr @class_name.256)
  %sel368 = call ptr @sel_registerName(ptr @sel_name.257)
  %msg_result369 = call ptr @objc_msgSend(ptr %cls367, ptr %sel368, i64 0)
  %sel370 = call ptr @sel_registerName(ptr @sel_name.258)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel370, { { double, double }, { double, double } } { { double, double } { double 0.000000e+00, double 3.000000e+00 }, { double, double } { double 1.400000e+02, double 4.800000e+01 } })
  %cls371 = call ptr @objc_getClass(ptr @class_name.260)
  %sel372 = call ptr @sel_registerName(ptr @sel_name.261)
  %msg_result373 = call ptr @objc_msgSend(ptr %cls371, ptr %sel372)
  %sel374 = call ptr @sel_registerName(ptr @sel_name.262)
  %msg_result375 = call ptr @objc_msgSend(ptr %msg_result373, ptr %sel374, ptr @text_content.259)
  %sel376 = call ptr @sel_registerName(ptr @sel_name.263)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel376, ptr %msg_result375, i64 0)
  %cls377 = call ptr @objc_getClass(ptr @class_name.264)
  %sel378 = call ptr @sel_registerName(ptr @sel_name.265)
  %msg_result379 = call ptr @objc_msgSend(ptr %cls377, ptr %sel378, double 0x3FD1515151515151, double 0x3FDA1A1A1A1A1A1A, double 0x3FEB9B9B9B9B9B9C, double 1.000000e+00)
  %sel380 = call ptr @sel_registerName(ptr @sel_name.266)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel380, ptr %msg_result379)
  %sel381 = call ptr @sel_registerName(ptr @sel_name.267)
  %msg_result382 = call ptr @objc_msgSend(ptr %msg_result369, ptr %sel381)
  %sel383 = call ptr @sel_registerName(ptr @sel_name.268)
  call void @objc_msgSend(ptr %msg_result382, ptr %sel383, double 1.200000e+01)
  %cls384 = call ptr @objc_getClass(ptr @class_name.269)
  %sel385 = call ptr @sel_registerName(ptr @sel_name.270)
  %msg_result386 = call ptr @objc_msgSend(ptr %cls384, ptr %sel385)
  %sel387 = call ptr @sel_registerName(ptr @sel_name.271)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel387, ptr %msg_result386, i64 0)
  %sel388 = call ptr @sel_registerName(ptr @sel_name.272)
  %sel389 = call ptr @sel_registerName(ptr @sel_name.273)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel389, ptr %0, ptr %sel388, i64 64)
  %sel390 = call ptr @sel_registerName(ptr @sel_name.274)
  call void @objc_msgSend(ptr %msg_result362, ptr %sel390, ptr %msg_result369)
  %cls391 = call ptr @objc_getClass(ptr @class_name.275)
  %sel392 = call ptr @sel_registerName(ptr @sel_name.276)
  %msg_result393 = call ptr @objc_msgSend(ptr %cls391, ptr %sel392, i64 0)
  %sel394 = call ptr @sel_registerName(ptr @sel_name.277)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel394, { { double, double }, { double, double } } { { double, double } { double 1.780000e+02, double 3.000000e+00 }, { double, double } { double 1.400000e+02, double 4.800000e+01 } })
  %cls395 = call ptr @objc_getClass(ptr @class_name.279)
  %sel396 = call ptr @sel_registerName(ptr @sel_name.280)
  %msg_result397 = call ptr @objc_msgSend(ptr %cls395, ptr %sel396)
  %sel398 = call ptr @sel_registerName(ptr @sel_name.281)
  %msg_result399 = call ptr @objc_msgSend(ptr %msg_result397, ptr %sel398, ptr @text_content.278)
  %sel400 = call ptr @sel_registerName(ptr @sel_name.282)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel400, ptr %msg_result399, i64 0)
  %cls401 = call ptr @objc_getClass(ptr @class_name.283)
  %sel402 = call ptr @sel_registerName(ptr @sel_name.284)
  %msg_result403 = call ptr @objc_msgSend(ptr %cls401, ptr %sel402, double 0x3FC1919191919192, double 0x3FE3737373737373, double 0x3FDB9B9B9B9B9B9C, double 1.000000e+00)
  %sel404 = call ptr @sel_registerName(ptr @sel_name.285)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel404, ptr %msg_result403)
  %sel405 = call ptr @sel_registerName(ptr @sel_name.286)
  %msg_result406 = call ptr @objc_msgSend(ptr %msg_result393, ptr %sel405)
  %sel407 = call ptr @sel_registerName(ptr @sel_name.287)
  call void @objc_msgSend(ptr %msg_result406, ptr %sel407, double 1.200000e+01)
  %cls408 = call ptr @objc_getClass(ptr @class_name.288)
  %sel409 = call ptr @sel_registerName(ptr @sel_name.289)
  %msg_result410 = call ptr @objc_msgSend(ptr %cls408, ptr %sel409)
  %sel411 = call ptr @sel_registerName(ptr @sel_name.290)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel411, ptr %msg_result410, i64 0)
  %sel412 = call ptr @sel_registerName(ptr @sel_name.291)
  %sel413 = call ptr @sel_registerName(ptr @sel_name.292)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel413, ptr %0, ptr %sel412, i64 64)
  %sel414 = call ptr @sel_registerName(ptr @sel_name.293)
  call void @objc_msgSend(ptr %msg_result362, ptr %sel414, ptr %msg_result393)
  %sel415 = call ptr @sel_registerName(ptr @sel_name.294)
  call void @objc_msgSend(ptr %msg_result369, ptr %sel415, i1 false)
  %sel416 = call ptr @sel_registerName(ptr @sel_name.295)
  %msg_result417 = call ptr @objc_msgSend(ptr %msg_result369, ptr %sel416)
  %sel418 = call ptr @sel_registerName(ptr @sel_name.296)
  %msg_result419 = call ptr @objc_msgSend(ptr %msg_result417, ptr %sel418, double 1.400000e+02)
  %sel420 = call ptr @sel_registerName(ptr @sel_name.297)
  call void @objc_msgSend(ptr %msg_result419, ptr %sel420, i1 true)
  %sel421 = call ptr @sel_registerName(ptr @sel_name.298)
  %msg_result422 = call ptr @objc_msgSend(ptr %msg_result369, ptr %sel421)
  %sel423 = call ptr @sel_registerName(ptr @sel_name.299)
  %msg_result424 = call ptr @objc_msgSend(ptr %msg_result422, ptr %sel423, double 4.800000e+01)
  %sel425 = call ptr @sel_registerName(ptr @sel_name.300)
  call void @objc_msgSend(ptr %msg_result424, ptr %sel425, i1 true)
  %sel426 = call ptr @sel_registerName(ptr @sel_name.301)
  %msg_result427 = call ptr @objc_msgSend(ptr %msg_result369, ptr %sel426)
  %sel428 = call ptr @sel_registerName(ptr @sel_name.302)
  %msg_result429 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel428)
  %sel430 = call ptr @sel_registerName(ptr @sel_name.303)
  %msg_result431 = call ptr @objc_msgSend(ptr %msg_result427, ptr %sel430, ptr %msg_result429, double 0.000000e+00)
  %sel432 = call ptr @sel_registerName(ptr @sel_name.304)
  call void @objc_msgSend(ptr %msg_result431, ptr %sel432, i1 true)
  %sel433 = call ptr @sel_registerName(ptr @sel_name.305)
  %msg_result434 = call ptr @objc_msgSend(ptr %msg_result369, ptr %sel433)
  %sel435 = call ptr @sel_registerName(ptr @sel_name.306)
  %msg_result436 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel435)
  %sel437 = call ptr @sel_registerName(ptr @sel_name.307)
  %msg_result438 = call ptr @objc_msgSend(ptr %msg_result434, ptr %sel437, ptr %msg_result436, double 0.000000e+00)
  %sel439 = call ptr @sel_registerName(ptr @sel_name.308)
  call void @objc_msgSend(ptr %msg_result438, ptr %sel439, i1 true)
  %sel440 = call ptr @sel_registerName(ptr @sel_name.309)
  call void @objc_msgSend(ptr %msg_result393, ptr %sel440, i1 false)
  %sel441 = call ptr @sel_registerName(ptr @sel_name.310)
  %msg_result442 = call ptr @objc_msgSend(ptr %msg_result393, ptr %sel441)
  %sel443 = call ptr @sel_registerName(ptr @sel_name.311)
  %msg_result444 = call ptr @objc_msgSend(ptr %msg_result442, ptr %sel443, double 1.400000e+02)
  %sel445 = call ptr @sel_registerName(ptr @sel_name.312)
  call void @objc_msgSend(ptr %msg_result444, ptr %sel445, i1 true)
  %sel446 = call ptr @sel_registerName(ptr @sel_name.313)
  %msg_result447 = call ptr @objc_msgSend(ptr %msg_result393, ptr %sel446)
  %sel448 = call ptr @sel_registerName(ptr @sel_name.314)
  %msg_result449 = call ptr @objc_msgSend(ptr %msg_result447, ptr %sel448, double 4.800000e+01)
  %sel450 = call ptr @sel_registerName(ptr @sel_name.315)
  call void @objc_msgSend(ptr %msg_result449, ptr %sel450, i1 true)
  %sel451 = call ptr @sel_registerName(ptr @sel_name.316)
  %msg_result452 = call ptr @objc_msgSend(ptr %msg_result393, ptr %sel451)
  %sel453 = call ptr @sel_registerName(ptr @sel_name.317)
  %msg_result454 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel453)
  %sel455 = call ptr @sel_registerName(ptr @sel_name.318)
  %msg_result456 = call ptr @objc_msgSend(ptr %msg_result452, ptr %sel455, ptr %msg_result454, double 0.000000e+00)
  %sel457 = call ptr @sel_registerName(ptr @sel_name.319)
  call void @objc_msgSend(ptr %msg_result456, ptr %sel457, i1 true)
  %sel458 = call ptr @sel_registerName(ptr @sel_name.320)
  %msg_result459 = call ptr @objc_msgSend(ptr %msg_result393, ptr %sel458)
  %sel460 = call ptr @sel_registerName(ptr @sel_name.321)
  %msg_result461 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel460)
  %sel462 = call ptr @sel_registerName(ptr @sel_name.322)
  %msg_result463 = call ptr @objc_msgSend(ptr %msg_result459, ptr %sel462, ptr %msg_result461, double -0.000000e+00)
  %sel464 = call ptr @sel_registerName(ptr @sel_name.323)
  call void @objc_msgSend(ptr %msg_result463, ptr %sel464, i1 true)
  %sel465 = call ptr @sel_registerName(ptr @sel_name.324)
  call void @objc_msgSend(ptr %msg_result362, ptr %sel465, i8 0)
  %sel466 = call ptr @sel_registerName(ptr @sel_name.325)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel466, ptr %msg_result362)
  %sel467 = call ptr @sel_registerName(ptr @sel_name.326)
  %msg_result468 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel467)
  %sel469 = call ptr @sel_registerName(ptr @sel_name.327)
  %msg_result470 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel469)
  %sel471 = call ptr @sel_registerName(ptr @sel_name.328)
  %msg_result472 = call ptr @objc_msgSend(ptr %msg_result468, ptr %sel471, ptr %msg_result470, double 1.000000e+00, double -3.200000e+01)
  %sel473 = call ptr @sel_registerName(ptr @sel_name.329)
  call void @objc_msgSend(ptr %msg_result472, ptr %sel473, i1 true)
  %sel474 = call ptr @sel_registerName(ptr @sel_name.330)
  %msg_result475 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel474)
  %sel476 = call ptr @sel_registerName(ptr @sel_name.331)
  %msg_result477 = call ptr @objc_msgSend(ptr %msg_result475, ptr %sel476, double 5.400000e+01)
  %sel478 = call ptr @sel_registerName(ptr @sel_name.332)
  call void @objc_msgSend(ptr %msg_result477, ptr %sel478, i1 true)
  %cls479 = call ptr @objc_getClass(ptr @class_name.333)
  %sel480 = call ptr @sel_registerName(ptr @sel_name.334)
  %msg_result481 = call ptr @objc_msgSend(ptr %cls479, ptr %sel480)
  %sel482 = call ptr @sel_registerName(ptr @sel_name.335)
  %msg_result483 = call ptr @objc_msgSend(ptr %msg_result481, ptr %sel482, { { double, double }, { double, double } } { { double, double } { double 3.200000e+01, double 6.640000e+02 }, { double, double } { double 3.180000e+02, double 1.500000e+02 } })
  %cls484 = call ptr @objc_getClass(ptr @class_name.336)
  %sel485 = call ptr @sel_registerName(ptr @sel_name.337)
  %msg_result486 = call ptr @objc_msgSend(ptr %cls484, ptr %sel485, double 0x3FED7D7D7D7D7D7D, double 0x3FEE3E3E3E3E3E3E, double 0x3FEF9F9F9F9F9FA0, double 1.000000e+00)
  %sel487 = call ptr @sel_registerName(ptr @sel_name.338)
  call void @objc_msgSend(ptr %msg_result483, ptr %sel487, ptr %msg_result486)
  %sel488 = call ptr @sel_registerName(ptr @sel_name.339)
  %msg_result489 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel488)
  %sel490 = call ptr @sel_registerName(ptr @sel_name.340)
  call void @objc_msgSend(ptr %msg_result489, ptr %sel490, double 1.600000e+01)
  %cls491 = call ptr @objc_getClass(ptr @class_name.341)
  %sel492 = call ptr @sel_registerName(ptr @sel_name.342)
  %msg_result493 = call ptr @objc_msgSend(ptr %cls491, ptr %sel492)
  %sel494 = call ptr @sel_registerName(ptr @sel_name.343)
  %msg_result495 = call ptr @objc_msgSend(ptr %msg_result493, ptr %sel494, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.800000e+01 }, { double, double } { double 2.820000e+02, double 1.680000e+01 } })
  %cls496 = call ptr @objc_getClass(ptr @class_name.345)
  %sel497 = call ptr @sel_registerName(ptr @sel_name.346)
  %msg_result498 = call ptr @objc_msgSend(ptr %cls496, ptr %sel497)
  %sel499 = call ptr @sel_registerName(ptr @sel_name.347)
  %msg_result500 = call ptr @objc_msgSend(ptr %msg_result498, ptr %sel499, ptr @text_content.344)
  %sel501 = call ptr @sel_registerName(ptr @sel_name.348)
  call void @objc_msgSend(ptr %msg_result495, ptr %sel501, ptr %msg_result500)
  %cls502 = call ptr @objc_getClass(ptr @class_name.349)
  %sel503 = call ptr @sel_registerName(ptr @sel_name.350)
  %msg_result504 = call ptr @objc_msgSend(ptr %cls502, ptr %sel503)
  %sel505 = call ptr @sel_registerName(ptr @sel_name.351)
  call void @objc_msgSend(ptr %msg_result495, ptr %sel505, ptr %msg_result504)
  %cls506 = call ptr @objc_getClass(ptr @class_name.352)
  %sel507 = call ptr @sel_registerName(ptr @sel_name.353)
  %msg_result508 = call ptr @objc_msgSend(ptr %cls506, ptr %sel507, double 1.400000e+01)
  %sel509 = call ptr @sel_registerName(ptr @sel_name.354)
  call void @objc_msgSend(ptr %msg_result495, ptr %sel509, ptr %msg_result508)
  %sel510 = call ptr @sel_registerName(ptr @sel_name.355)
  call void @objc_msgSend(ptr %msg_result483, ptr %sel510, ptr %msg_result495)
  %cls511 = call ptr @objc_getClass(ptr @class_name.356)
  %sel512 = call ptr @sel_registerName(ptr @sel_name.357)
  %msg_result513 = call ptr @objc_msgSend(ptr %cls511, ptr %sel512)
  %sel514 = call ptr @sel_registerName(ptr @sel_name.358)
  %msg_result515 = call ptr @objc_msgSend(ptr %msg_result513, ptr %sel514, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.056000e+02 }, { double, double } { double 2.820000e+02, double 2.640000e+01 } })
  %cls516 = call ptr @objc_getClass(ptr @class_name.360)
  %sel517 = call ptr @sel_registerName(ptr @sel_name.361)
  %msg_result518 = call ptr @objc_msgSend(ptr %cls516, ptr %sel517)
  %sel519 = call ptr @sel_registerName(ptr @sel_name.362)
  %msg_result520 = call ptr @objc_msgSend(ptr %msg_result518, ptr %sel519, ptr @text_content.359)
  %sel521 = call ptr @sel_registerName(ptr @sel_name.363)
  call void @objc_msgSend(ptr %msg_result515, ptr %sel521, ptr %msg_result520)
  %cls522 = call ptr @objc_getClass(ptr @class_name.364)
  %sel523 = call ptr @sel_registerName(ptr @sel_name.365)
  %msg_result524 = call ptr @objc_msgSend(ptr %cls522, ptr %sel523, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel525 = call ptr @sel_registerName(ptr @sel_name.366)
  call void @objc_msgSend(ptr %msg_result515, ptr %sel525, ptr %msg_result524)
  %cls526 = call ptr @objc_getClass(ptr @class_name.367)
  %sel527 = call ptr @sel_registerName(ptr @sel_name.368)
  %msg_result528 = call ptr @objc_msgSend(ptr %cls526, ptr %sel527, double 2.200000e+01)
  %sel529 = call ptr @sel_registerName(ptr @sel_name.369)
  call void @objc_msgSend(ptr %msg_result515, ptr %sel529, ptr %msg_result528)
  %sel530 = call ptr @sel_registerName(ptr @sel_name.370)
  call void @objc_msgSend(ptr %msg_result483, ptr %sel530, ptr %msg_result515)
  %sel531 = call ptr @sel_registerName(ptr @sel_name.371)
  call void @objc_msgSend(ptr %msg_result495, ptr %sel531, i1 false)
  %sel532 = call ptr @sel_registerName(ptr @sel_name.372)
  %msg_result533 = call ptr @objc_msgSend(ptr %msg_result495, ptr %sel532)
  %sel534 = call ptr @sel_registerName(ptr @sel_name.373)
  %msg_result535 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel534)
  %sel536 = call ptr @sel_registerName(ptr @sel_name.374)
  %msg_result537 = call ptr @objc_msgSend(ptr %msg_result533, ptr %sel536, ptr %msg_result535, double 1.800000e+01)
  %sel538 = call ptr @sel_registerName(ptr @sel_name.375)
  call void @objc_msgSend(ptr %msg_result537, ptr %sel538, i1 true)
  %sel539 = call ptr @sel_registerName(ptr @sel_name.376)
  %msg_result540 = call ptr @objc_msgSend(ptr %msg_result495, ptr %sel539)
  %sel541 = call ptr @sel_registerName(ptr @sel_name.377)
  %msg_result542 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel541)
  %sel543 = call ptr @sel_registerName(ptr @sel_name.378)
  %msg_result544 = call ptr @objc_msgSend(ptr %msg_result540, ptr %sel543, ptr %msg_result542, double -1.800000e+01)
  %sel545 = call ptr @sel_registerName(ptr @sel_name.379)
  call void @objc_msgSend(ptr %msg_result544, ptr %sel545, i1 true)
  %sel546 = call ptr @sel_registerName(ptr @sel_name.380)
  %msg_result547 = call ptr @objc_msgSend(ptr %msg_result495, ptr %sel546)
  %sel548 = call ptr @sel_registerName(ptr @sel_name.381)
  %msg_result549 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel548)
  %sel550 = call ptr @sel_registerName(ptr @sel_name.382)
  %msg_result551 = call ptr @objc_msgSend(ptr %msg_result547, ptr %sel550, ptr %msg_result549, double 1.800000e+01)
  %sel552 = call ptr @sel_registerName(ptr @sel_name.383)
  call void @objc_msgSend(ptr %msg_result551, ptr %sel552, i1 true)
  %sel553 = call ptr @sel_registerName(ptr @sel_name.384)
  call void @objc_msgSend(ptr %msg_result515, ptr %sel553, i1 false)
  %sel554 = call ptr @sel_registerName(ptr @sel_name.385)
  %msg_result555 = call ptr @objc_msgSend(ptr %msg_result515, ptr %sel554)
  %sel556 = call ptr @sel_registerName(ptr @sel_name.386)
  %msg_result557 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel556)
  %sel558 = call ptr @sel_registerName(ptr @sel_name.387)
  %msg_result559 = call ptr @objc_msgSend(ptr %msg_result555, ptr %sel558, ptr %msg_result557, double 1.800000e+01)
  %sel560 = call ptr @sel_registerName(ptr @sel_name.388)
  call void @objc_msgSend(ptr %msg_result559, ptr %sel560, i1 true)
  %sel561 = call ptr @sel_registerName(ptr @sel_name.389)
  %msg_result562 = call ptr @objc_msgSend(ptr %msg_result515, ptr %sel561)
  %sel563 = call ptr @sel_registerName(ptr @sel_name.390)
  %msg_result564 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel563)
  %sel565 = call ptr @sel_registerName(ptr @sel_name.391)
  %msg_result566 = call ptr @objc_msgSend(ptr %msg_result562, ptr %sel565, ptr %msg_result564, double -1.800000e+01)
  %sel567 = call ptr @sel_registerName(ptr @sel_name.392)
  call void @objc_msgSend(ptr %msg_result566, ptr %sel567, i1 true)
  %sel568 = call ptr @sel_registerName(ptr @sel_name.393)
  %msg_result569 = call ptr @objc_msgSend(ptr %msg_result515, ptr %sel568)
  %sel570 = call ptr @sel_registerName(ptr @sel_name.394)
  %msg_result571 = call ptr @objc_msgSend(ptr %msg_result483, ptr %sel570)
  %sel572 = call ptr @sel_registerName(ptr @sel_name.395)
  %msg_result573 = call ptr @objc_msgSend(ptr %msg_result569, ptr %sel572, ptr %msg_result571, double -1.800000e+01)
  %sel574 = call ptr @sel_registerName(ptr @sel_name.396)
  call void @objc_msgSend(ptr %msg_result573, ptr %sel574, i1 true)
  %sel575 = call ptr @sel_registerName(ptr @sel_name.397)
  call void @objc_msgSend(ptr %msg_result483, ptr %sel575, i8 0)
  store ptr %msg_result483, ptr @g_conditional_true_0, align 8
  store ptr null, ptr @g_conditional_false_0, align 8
  store i8 0, ptr @g_conditional_state_0, align 1
  store ptr %msg_result131, ptr @g_conditional_stack_0, align 8
  %conditional_true_for_layout = load ptr, ptr @g_conditional_true_0, align 8
  %conditional_false_for_layout = load ptr, ptr @g_conditional_false_0, align 8
  %sel576 = call ptr @sel_registerName(ptr @sel_name.398)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel576, ptr %conditional_true_for_layout)
  %sel577 = call ptr @sel_registerName(ptr @sel_name.399)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel577, ptr %conditional_false_for_layout)
  %sel578 = call ptr @sel_registerName(ptr @sel_name.400)
  call void @objc_msgSend(ptr %conditional_true_for_layout, ptr %sel578, i8 1)
  %sel579 = call ptr @sel_registerName(ptr @sel_name.401)
  call void @objc_msgSend(ptr %conditional_false_for_layout, ptr %sel579, i8 0)
  %sel580 = call ptr @sel_registerName(ptr @sel_name.402)
  %msg_result581 = call ptr @objc_msgSend(ptr %conditional_true_for_layout, ptr %sel580)
  %sel582 = call ptr @sel_registerName(ptr @sel_name.403)
  %msg_result583 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel582)
  %sel584 = call ptr @sel_registerName(ptr @sel_name.404)
  %msg_result585 = call ptr @objc_msgSend(ptr %msg_result581, ptr %sel584, ptr %msg_result583, double 1.000000e+00, double -3.200000e+01)
  %sel586 = call ptr @sel_registerName(ptr @sel_name.405)
  call void @objc_msgSend(ptr %msg_result585, ptr %sel586, i1 true)
  %sel587 = call ptr @sel_registerName(ptr @sel_name.406)
  %msg_result588 = call ptr @objc_msgSend(ptr %conditional_true_for_layout, ptr %sel587)
  %sel589 = call ptr @sel_registerName(ptr @sel_name.407)
  %msg_result590 = call ptr @objc_msgSend(ptr %msg_result588, ptr %sel589, double 1.500000e+02)
  %sel591 = call ptr @sel_registerName(ptr @sel_name.408)
  call void @objc_msgSend(ptr %msg_result590, ptr %sel591, i1 true)
  %cls592 = call ptr @objc_getClass(ptr @class_name.409)
  %sel593 = call ptr @sel_registerName(ptr @sel_name.410)
  %msg_result594 = call ptr @objc_msgSend(ptr %cls592, ptr %sel593)
  %sel595 = call ptr @sel_registerName(ptr @sel_name.411)
  %msg_result596 = call ptr @objc_msgSend(ptr %msg_result594, ptr %sel595, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 4.960000e+02 }, { double, double } { double 3.180000e+02, double 1.680000e+01 } })
  %cls597 = call ptr @objc_getClass(ptr @class_name.413)
  %sel598 = call ptr @sel_registerName(ptr @sel_name.414)
  %msg_result599 = call ptr @objc_msgSend(ptr %cls597, ptr %sel598)
  %sel600 = call ptr @sel_registerName(ptr @sel_name.415)
  %msg_result601 = call ptr @objc_msgSend(ptr %msg_result599, ptr %sel600, ptr @text_content.412)
  %sel602 = call ptr @sel_registerName(ptr @sel_name.416)
  call void @objc_msgSend(ptr %msg_result596, ptr %sel602, ptr %msg_result601)
  %cls603 = call ptr @objc_getClass(ptr @class_name.417)
  %sel604 = call ptr @sel_registerName(ptr @sel_name.418)
  %msg_result605 = call ptr @objc_msgSend(ptr %cls603, ptr %sel604)
  %sel606 = call ptr @sel_registerName(ptr @sel_name.419)
  call void @objc_msgSend(ptr %msg_result596, ptr %sel606, ptr %msg_result605)
  %cls607 = call ptr @objc_getClass(ptr @class_name.420)
  %sel608 = call ptr @sel_registerName(ptr @sel_name.421)
  %msg_result609 = call ptr @objc_msgSend(ptr %cls607, ptr %sel608, double 1.400000e+01)
  %sel610 = call ptr @sel_registerName(ptr @sel_name.422)
  call void @objc_msgSend(ptr %msg_result596, ptr %sel610, ptr %msg_result609)
  %sel611 = call ptr @sel_registerName(ptr @sel_name.423)
  call void @objc_msgSend(ptr %msg_result596, ptr %sel611, i8 0)
  %sel612 = call ptr @sel_registerName(ptr @sel_name.424)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel612, ptr %msg_result596)
  %cls613 = call ptr @objc_getClass(ptr @class_name.425)
  %sel614 = call ptr @sel_registerName(ptr @sel_name.426)
  %msg_result615 = call ptr @objc_msgSend(ptr %cls613, ptr %sel614)
  %sel616 = call ptr @sel_registerName(ptr @sel_name.427)
  %msg_result617 = call ptr @objc_msgSend(ptr %msg_result615, ptr %sel616, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 5.268000e+02 }, { double, double } { double 3.180000e+02, double 0x4035999999999999 } })
  %cls618 = call ptr @objc_getClass(ptr @class_name.429)
  %sel619 = call ptr @sel_registerName(ptr @sel_name.430)
  %msg_result620 = call ptr @objc_msgSend(ptr %cls618, ptr %sel619)
  %sel621 = call ptr @sel_registerName(ptr @sel_name.431)
  %msg_result622 = call ptr @objc_msgSend(ptr %msg_result620, ptr %sel621, ptr @text_content.428)
  %sel623 = call ptr @sel_registerName(ptr @sel_name.432)
  call void @objc_msgSend(ptr %msg_result617, ptr %sel623, ptr %msg_result622)
  %cls624 = call ptr @objc_getClass(ptr @class_name.433)
  %sel625 = call ptr @sel_registerName(ptr @sel_name.434)
  %msg_result626 = call ptr @objc_msgSend(ptr %cls624, ptr %sel625, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel627 = call ptr @sel_registerName(ptr @sel_name.435)
  call void @objc_msgSend(ptr %msg_result617, ptr %sel627, ptr %msg_result626)
  %cls628 = call ptr @objc_getClass(ptr @class_name.436)
  %sel629 = call ptr @sel_registerName(ptr @sel_name.437)
  %msg_result630 = call ptr @objc_msgSend(ptr %cls628, ptr %sel629, double 1.800000e+01)
  %sel631 = call ptr @sel_registerName(ptr @sel_name.438)
  call void @objc_msgSend(ptr %msg_result617, ptr %sel631, ptr %msg_result630)
  %sel632 = call ptr @sel_registerName(ptr @sel_name.439)
  call void @objc_msgSend(ptr %msg_result617, ptr %sel632, i8 0)
  %sel633 = call ptr @sel_registerName(ptr @sel_name.440)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel633, ptr %msg_result617)
  %cls634 = call ptr @objc_getClass(ptr @class_name.441)
  %sel635 = call ptr @sel_registerName(ptr @sel_name.442)
  %msg_result636 = call ptr @objc_msgSend(ptr %cls634, ptr %sel635)
  %sel637 = call ptr @sel_registerName(ptr @sel_name.443)
  %msg_result638 = call ptr @objc_msgSend(ptr %msg_result636, ptr %sel637, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 5.624000e+02 }, { double, double } { double 3.180000e+02, double 1.900000e+02 } })
  %cls639 = call ptr @objc_getClass(ptr @class_name.444)
  %sel640 = call ptr @sel_registerName(ptr @sel_name.445)
  %msg_result641 = call ptr @objc_msgSend(ptr %cls639, ptr %sel640, double 0x3FED7D7D7D7D7D7D, double 0x3FEE3E3E3E3E3E3E, double 0x3FEF9F9F9F9F9FA0, double 1.000000e+00)
  %sel642 = call ptr @sel_registerName(ptr @sel_name.446)
  call void @objc_msgSend(ptr %msg_result638, ptr %sel642, ptr %msg_result641)
  %sel643 = call ptr @sel_registerName(ptr @sel_name.447)
  %msg_result644 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel643)
  %sel645 = call ptr @sel_registerName(ptr @sel_name.448)
  call void @objc_msgSend(ptr %msg_result644, ptr %sel645, double 1.600000e+01)
  %cls646 = call ptr @objc_getClass(ptr @class_name.449)
  %sel647 = call ptr @sel_registerName(ptr @sel_name.450)
  %msg_result648 = call ptr @objc_msgSend(ptr %cls646, ptr %sel647)
  %sel649 = call ptr @sel_registerName(ptr @sel_name.451)
  %msg_result650 = call ptr @objc_msgSend(ptr %msg_result648, ptr %sel649, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.800000e+01 }, { double, double } { double 2.820000e+02, double 1.680000e+01 } })
  %cls651 = call ptr @objc_getClass(ptr @class_name.453)
  %sel652 = call ptr @sel_registerName(ptr @sel_name.454)
  %msg_result653 = call ptr @objc_msgSend(ptr %cls651, ptr %sel652)
  %sel654 = call ptr @sel_registerName(ptr @sel_name.455)
  %msg_result655 = call ptr @objc_msgSend(ptr %msg_result653, ptr %sel654, ptr @text_content.452)
  %sel656 = call ptr @sel_registerName(ptr @sel_name.456)
  call void @objc_msgSend(ptr %msg_result650, ptr %sel656, ptr %msg_result655)
  %cls657 = call ptr @objc_getClass(ptr @class_name.457)
  %sel658 = call ptr @sel_registerName(ptr @sel_name.458)
  %msg_result659 = call ptr @objc_msgSend(ptr %cls657, ptr %sel658)
  %sel660 = call ptr @sel_registerName(ptr @sel_name.459)
  call void @objc_msgSend(ptr %msg_result650, ptr %sel660, ptr %msg_result659)
  %cls661 = call ptr @objc_getClass(ptr @class_name.460)
  %sel662 = call ptr @sel_registerName(ptr @sel_name.461)
  %msg_result663 = call ptr @objc_msgSend(ptr %cls661, ptr %sel662, double 1.400000e+01)
  %sel664 = call ptr @sel_registerName(ptr @sel_name.462)
  call void @objc_msgSend(ptr %msg_result650, ptr %sel664, ptr %msg_result663)
  %sel665 = call ptr @sel_registerName(ptr @sel_name.463)
  call void @objc_msgSend(ptr %msg_result638, ptr %sel665, ptr %msg_result650)
  %cls666 = call ptr @objc_getClass(ptr @class_name.464)
  %sel667 = call ptr @sel_registerName(ptr @sel_name.465)
  %msg_result668 = call ptr @objc_msgSend(ptr %cls666, ptr %sel667)
  %sel669 = call ptr @sel_registerName(ptr @sel_name.466)
  %msg_result670 = call ptr @objc_msgSend(ptr %msg_result668, ptr %sel669, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.456000e+02 }, { double, double } { double 2.820000e+02, double 2.640000e+01 } })
  %cls671 = call ptr @objc_getClass(ptr @class_name.468)
  %sel672 = call ptr @sel_registerName(ptr @sel_name.469)
  %msg_result673 = call ptr @objc_msgSend(ptr %cls671, ptr %sel672)
  %sel674 = call ptr @sel_registerName(ptr @sel_name.470)
  %msg_result675 = call ptr @objc_msgSend(ptr %msg_result673, ptr %sel674, ptr @text_content.467)
  %sel676 = call ptr @sel_registerName(ptr @sel_name.471)
  call void @objc_msgSend(ptr %msg_result670, ptr %sel676, ptr %msg_result675)
  %cls677 = call ptr @objc_getClass(ptr @class_name.472)
  %sel678 = call ptr @sel_registerName(ptr @sel_name.473)
  %msg_result679 = call ptr @objc_msgSend(ptr %cls677, ptr %sel678, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel680 = call ptr @sel_registerName(ptr @sel_name.474)
  call void @objc_msgSend(ptr %msg_result670, ptr %sel680, ptr %msg_result679)
  %cls681 = call ptr @objc_getClass(ptr @class_name.475)
  %sel682 = call ptr @sel_registerName(ptr @sel_name.476)
  %msg_result683 = call ptr @objc_msgSend(ptr %cls681, ptr %sel682, double 2.200000e+01)
  %sel684 = call ptr @sel_registerName(ptr @sel_name.477)
  call void @objc_msgSend(ptr %msg_result670, ptr %sel684, ptr %msg_result683)
  %sel685 = call ptr @sel_registerName(ptr @sel_name.478)
  call void @objc_msgSend(ptr %msg_result638, ptr %sel685, ptr %msg_result670)
  %sel686 = call ptr @sel_registerName(ptr @sel_name.479)
  call void @objc_msgSend(ptr %msg_result650, ptr %sel686, i1 false)
  %sel687 = call ptr @sel_registerName(ptr @sel_name.480)
  %msg_result688 = call ptr @objc_msgSend(ptr %msg_result650, ptr %sel687)
  %sel689 = call ptr @sel_registerName(ptr @sel_name.481)
  %msg_result690 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel689)
  %sel691 = call ptr @sel_registerName(ptr @sel_name.482)
  %msg_result692 = call ptr @objc_msgSend(ptr %msg_result688, ptr %sel691, ptr %msg_result690, double 1.800000e+01)
  %sel693 = call ptr @sel_registerName(ptr @sel_name.483)
  call void @objc_msgSend(ptr %msg_result692, ptr %sel693, i1 true)
  %sel694 = call ptr @sel_registerName(ptr @sel_name.484)
  %msg_result695 = call ptr @objc_msgSend(ptr %msg_result650, ptr %sel694)
  %sel696 = call ptr @sel_registerName(ptr @sel_name.485)
  %msg_result697 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel696)
  %sel698 = call ptr @sel_registerName(ptr @sel_name.486)
  %msg_result699 = call ptr @objc_msgSend(ptr %msg_result695, ptr %sel698, ptr %msg_result697, double -1.800000e+01)
  %sel700 = call ptr @sel_registerName(ptr @sel_name.487)
  call void @objc_msgSend(ptr %msg_result699, ptr %sel700, i1 true)
  %sel701 = call ptr @sel_registerName(ptr @sel_name.488)
  %msg_result702 = call ptr @objc_msgSend(ptr %msg_result650, ptr %sel701)
  %sel703 = call ptr @sel_registerName(ptr @sel_name.489)
  %msg_result704 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel703)
  %sel705 = call ptr @sel_registerName(ptr @sel_name.490)
  %msg_result706 = call ptr @objc_msgSend(ptr %msg_result702, ptr %sel705, ptr %msg_result704, double 1.800000e+01)
  %sel707 = call ptr @sel_registerName(ptr @sel_name.491)
  call void @objc_msgSend(ptr %msg_result706, ptr %sel707, i1 true)
  %sel708 = call ptr @sel_registerName(ptr @sel_name.492)
  call void @objc_msgSend(ptr %msg_result670, ptr %sel708, i1 false)
  %sel709 = call ptr @sel_registerName(ptr @sel_name.493)
  %msg_result710 = call ptr @objc_msgSend(ptr %msg_result670, ptr %sel709)
  %sel711 = call ptr @sel_registerName(ptr @sel_name.494)
  %msg_result712 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel711)
  %sel713 = call ptr @sel_registerName(ptr @sel_name.495)
  %msg_result714 = call ptr @objc_msgSend(ptr %msg_result710, ptr %sel713, ptr %msg_result712, double 1.800000e+01)
  %sel715 = call ptr @sel_registerName(ptr @sel_name.496)
  call void @objc_msgSend(ptr %msg_result714, ptr %sel715, i1 true)
  %sel716 = call ptr @sel_registerName(ptr @sel_name.497)
  %msg_result717 = call ptr @objc_msgSend(ptr %msg_result670, ptr %sel716)
  %sel718 = call ptr @sel_registerName(ptr @sel_name.498)
  %msg_result719 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel718)
  %sel720 = call ptr @sel_registerName(ptr @sel_name.499)
  %msg_result721 = call ptr @objc_msgSend(ptr %msg_result717, ptr %sel720, ptr %msg_result719, double -1.800000e+01)
  %sel722 = call ptr @sel_registerName(ptr @sel_name.500)
  call void @objc_msgSend(ptr %msg_result721, ptr %sel722, i1 true)
  %sel723 = call ptr @sel_registerName(ptr @sel_name.501)
  %msg_result724 = call ptr @objc_msgSend(ptr %msg_result670, ptr %sel723)
  %sel725 = call ptr @sel_registerName(ptr @sel_name.502)
  %msg_result726 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel725)
  %sel727 = call ptr @sel_registerName(ptr @sel_name.503)
  %msg_result728 = call ptr @objc_msgSend(ptr %msg_result724, ptr %sel727, ptr %msg_result726, double -1.800000e+01)
  %sel729 = call ptr @sel_registerName(ptr @sel_name.504)
  call void @objc_msgSend(ptr %msg_result728, ptr %sel729, i1 true)
  %sel730 = call ptr @sel_registerName(ptr @sel_name.505)
  call void @objc_msgSend(ptr %msg_result638, ptr %sel730, i8 0)
  %sel731 = call ptr @sel_registerName(ptr @sel_name.506)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel731, ptr %msg_result638)
  %sel732 = call ptr @sel_registerName(ptr @sel_name.507)
  %msg_result733 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel732)
  %sel734 = call ptr @sel_registerName(ptr @sel_name.508)
  %msg_result735 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel734)
  %sel736 = call ptr @sel_registerName(ptr @sel_name.509)
  %msg_result737 = call ptr @objc_msgSend(ptr %msg_result733, ptr %sel736, ptr %msg_result735, double 1.000000e+00, double -3.200000e+01)
  %sel738 = call ptr @sel_registerName(ptr @sel_name.510)
  call void @objc_msgSend(ptr %msg_result737, ptr %sel738, i1 true)
  %sel739 = call ptr @sel_registerName(ptr @sel_name.511)
  %msg_result740 = call ptr @objc_msgSend(ptr %msg_result638, ptr %sel739)
  %sel741 = call ptr @sel_registerName(ptr @sel_name.512)
  %msg_result742 = call ptr @objc_msgSend(ptr %msg_result740, ptr %sel741, double 1.900000e+02)
  %sel743 = call ptr @sel_registerName(ptr @sel_name.513)
  call void @objc_msgSend(ptr %msg_result742, ptr %sel743, i1 true)
  %cls744 = call ptr @objc_getClass(ptr @class_name.514)
  %sel745 = call ptr @sel_registerName(ptr @sel_name.515)
  %msg_result746 = call ptr @objc_msgSend(ptr %cls744, ptr %sel745)
  %sel747 = call ptr @sel_registerName(ptr @sel_name.516)
  %msg_result748 = call ptr @objc_msgSend(ptr %msg_result746, ptr %sel747, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 0x4087F33333333333 }, { double, double } { double 3.180000e+02, double 1.900000e+02 } })
  %cls749 = call ptr @objc_getClass(ptr @class_name.517)
  %sel750 = call ptr @sel_registerName(ptr @sel_name.518)
  %msg_result751 = call ptr @objc_msgSend(ptr %cls749, ptr %sel750, double 0x3FED1D1D1D1D1D1D, double 0x3FEEDEDEDEDEDEDF, double 0x3FEDFDFDFDFDFDFE, double 1.000000e+00)
  %sel752 = call ptr @sel_registerName(ptr @sel_name.519)
  call void @objc_msgSend(ptr %msg_result748, ptr %sel752, ptr %msg_result751)
  %sel753 = call ptr @sel_registerName(ptr @sel_name.520)
  %msg_result754 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel753)
  %sel755 = call ptr @sel_registerName(ptr @sel_name.521)
  call void @objc_msgSend(ptr %msg_result754, ptr %sel755, double 1.600000e+01)
  %cls756 = call ptr @objc_getClass(ptr @class_name.522)
  %sel757 = call ptr @sel_registerName(ptr @sel_name.523)
  %msg_result758 = call ptr @objc_msgSend(ptr %cls756, ptr %sel757)
  %sel759 = call ptr @sel_registerName(ptr @sel_name.524)
  %msg_result760 = call ptr @objc_msgSend(ptr %msg_result758, ptr %sel759, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.800000e+01 }, { double, double } { double 2.820000e+02, double 1.680000e+01 } })
  %cls761 = call ptr @objc_getClass(ptr @class_name.526)
  %sel762 = call ptr @sel_registerName(ptr @sel_name.527)
  %msg_result763 = call ptr @objc_msgSend(ptr %cls761, ptr %sel762)
  %sel764 = call ptr @sel_registerName(ptr @sel_name.528)
  %msg_result765 = call ptr @objc_msgSend(ptr %msg_result763, ptr %sel764, ptr @text_content.525)
  %sel766 = call ptr @sel_registerName(ptr @sel_name.529)
  call void @objc_msgSend(ptr %msg_result760, ptr %sel766, ptr %msg_result765)
  %cls767 = call ptr @objc_getClass(ptr @class_name.530)
  %sel768 = call ptr @sel_registerName(ptr @sel_name.531)
  %msg_result769 = call ptr @objc_msgSend(ptr %cls767, ptr %sel768)
  %sel770 = call ptr @sel_registerName(ptr @sel_name.532)
  call void @objc_msgSend(ptr %msg_result760, ptr %sel770, ptr %msg_result769)
  %cls771 = call ptr @objc_getClass(ptr @class_name.533)
  %sel772 = call ptr @sel_registerName(ptr @sel_name.534)
  %msg_result773 = call ptr @objc_msgSend(ptr %cls771, ptr %sel772, double 1.400000e+01)
  %sel774 = call ptr @sel_registerName(ptr @sel_name.535)
  call void @objc_msgSend(ptr %msg_result760, ptr %sel774, ptr %msg_result773)
  %sel775 = call ptr @sel_registerName(ptr @sel_name.536)
  call void @objc_msgSend(ptr %msg_result748, ptr %sel775, ptr %msg_result760)
  %cls776 = call ptr @objc_getClass(ptr @class_name.537)
  %sel777 = call ptr @sel_registerName(ptr @sel_name.538)
  %msg_result778 = call ptr @objc_msgSend(ptr %cls776, ptr %sel777)
  %sel779 = call ptr @sel_registerName(ptr @sel_name.539)
  %msg_result780 = call ptr @objc_msgSend(ptr %msg_result778, ptr %sel779, { { double, double }, { double, double } } { { double, double } { double 1.800000e+01, double 1.456000e+02 }, { double, double } { double 2.820000e+02, double 2.640000e+01 } })
  %cls781 = call ptr @objc_getClass(ptr @class_name.541)
  %sel782 = call ptr @sel_registerName(ptr @sel_name.542)
  %msg_result783 = call ptr @objc_msgSend(ptr %cls781, ptr %sel782)
  %sel784 = call ptr @sel_registerName(ptr @sel_name.543)
  %msg_result785 = call ptr @objc_msgSend(ptr %msg_result783, ptr %sel784, ptr @text_content.540)
  %sel786 = call ptr @sel_registerName(ptr @sel_name.544)
  call void @objc_msgSend(ptr %msg_result780, ptr %sel786, ptr %msg_result785)
  %cls787 = call ptr @objc_getClass(ptr @class_name.545)
  %sel788 = call ptr @sel_registerName(ptr @sel_name.546)
  %msg_result789 = call ptr @objc_msgSend(ptr %cls787, ptr %sel788, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel790 = call ptr @sel_registerName(ptr @sel_name.547)
  call void @objc_msgSend(ptr %msg_result780, ptr %sel790, ptr %msg_result789)
  %cls791 = call ptr @objc_getClass(ptr @class_name.548)
  %sel792 = call ptr @sel_registerName(ptr @sel_name.549)
  %msg_result793 = call ptr @objc_msgSend(ptr %cls791, ptr %sel792, double 2.200000e+01)
  %sel794 = call ptr @sel_registerName(ptr @sel_name.550)
  call void @objc_msgSend(ptr %msg_result780, ptr %sel794, ptr %msg_result793)
  %sel795 = call ptr @sel_registerName(ptr @sel_name.551)
  call void @objc_msgSend(ptr %msg_result748, ptr %sel795, ptr %msg_result780)
  %sel796 = call ptr @sel_registerName(ptr @sel_name.552)
  call void @objc_msgSend(ptr %msg_result760, ptr %sel796, i1 false)
  %sel797 = call ptr @sel_registerName(ptr @sel_name.553)
  %msg_result798 = call ptr @objc_msgSend(ptr %msg_result760, ptr %sel797)
  %sel799 = call ptr @sel_registerName(ptr @sel_name.554)
  %msg_result800 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel799)
  %sel801 = call ptr @sel_registerName(ptr @sel_name.555)
  %msg_result802 = call ptr @objc_msgSend(ptr %msg_result798, ptr %sel801, ptr %msg_result800, double 1.800000e+01)
  %sel803 = call ptr @sel_registerName(ptr @sel_name.556)
  call void @objc_msgSend(ptr %msg_result802, ptr %sel803, i1 true)
  %sel804 = call ptr @sel_registerName(ptr @sel_name.557)
  %msg_result805 = call ptr @objc_msgSend(ptr %msg_result760, ptr %sel804)
  %sel806 = call ptr @sel_registerName(ptr @sel_name.558)
  %msg_result807 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel806)
  %sel808 = call ptr @sel_registerName(ptr @sel_name.559)
  %msg_result809 = call ptr @objc_msgSend(ptr %msg_result805, ptr %sel808, ptr %msg_result807, double -1.800000e+01)
  %sel810 = call ptr @sel_registerName(ptr @sel_name.560)
  call void @objc_msgSend(ptr %msg_result809, ptr %sel810, i1 true)
  %sel811 = call ptr @sel_registerName(ptr @sel_name.561)
  %msg_result812 = call ptr @objc_msgSend(ptr %msg_result760, ptr %sel811)
  %sel813 = call ptr @sel_registerName(ptr @sel_name.562)
  %msg_result814 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel813)
  %sel815 = call ptr @sel_registerName(ptr @sel_name.563)
  %msg_result816 = call ptr @objc_msgSend(ptr %msg_result812, ptr %sel815, ptr %msg_result814, double 1.800000e+01)
  %sel817 = call ptr @sel_registerName(ptr @sel_name.564)
  call void @objc_msgSend(ptr %msg_result816, ptr %sel817, i1 true)
  %sel818 = call ptr @sel_registerName(ptr @sel_name.565)
  call void @objc_msgSend(ptr %msg_result780, ptr %sel818, i1 false)
  %sel819 = call ptr @sel_registerName(ptr @sel_name.566)
  %msg_result820 = call ptr @objc_msgSend(ptr %msg_result780, ptr %sel819)
  %sel821 = call ptr @sel_registerName(ptr @sel_name.567)
  %msg_result822 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel821)
  %sel823 = call ptr @sel_registerName(ptr @sel_name.568)
  %msg_result824 = call ptr @objc_msgSend(ptr %msg_result820, ptr %sel823, ptr %msg_result822, double 1.800000e+01)
  %sel825 = call ptr @sel_registerName(ptr @sel_name.569)
  call void @objc_msgSend(ptr %msg_result824, ptr %sel825, i1 true)
  %sel826 = call ptr @sel_registerName(ptr @sel_name.570)
  %msg_result827 = call ptr @objc_msgSend(ptr %msg_result780, ptr %sel826)
  %sel828 = call ptr @sel_registerName(ptr @sel_name.571)
  %msg_result829 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel828)
  %sel830 = call ptr @sel_registerName(ptr @sel_name.572)
  %msg_result831 = call ptr @objc_msgSend(ptr %msg_result827, ptr %sel830, ptr %msg_result829, double -1.800000e+01)
  %sel832 = call ptr @sel_registerName(ptr @sel_name.573)
  call void @objc_msgSend(ptr %msg_result831, ptr %sel832, i1 true)
  %sel833 = call ptr @sel_registerName(ptr @sel_name.574)
  %msg_result834 = call ptr @objc_msgSend(ptr %msg_result780, ptr %sel833)
  %sel835 = call ptr @sel_registerName(ptr @sel_name.575)
  %msg_result836 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel835)
  %sel837 = call ptr @sel_registerName(ptr @sel_name.576)
  %msg_result838 = call ptr @objc_msgSend(ptr %msg_result834, ptr %sel837, ptr %msg_result836, double -1.800000e+01)
  %sel839 = call ptr @sel_registerName(ptr @sel_name.577)
  call void @objc_msgSend(ptr %msg_result838, ptr %sel839, i1 true)
  %sel840 = call ptr @sel_registerName(ptr @sel_name.578)
  call void @objc_msgSend(ptr %msg_result748, ptr %sel840, i8 0)
  %sel841 = call ptr @sel_registerName(ptr @sel_name.579)
  call void @objc_msgSend(ptr %msg_result131, ptr %sel841, ptr %msg_result748)
  %sel842 = call ptr @sel_registerName(ptr @sel_name.580)
  %msg_result843 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel842)
  %sel844 = call ptr @sel_registerName(ptr @sel_name.581)
  %msg_result845 = call ptr @objc_msgSend(ptr %msg_result131, ptr %sel844)
  %sel846 = call ptr @sel_registerName(ptr @sel_name.582)
  %msg_result847 = call ptr @objc_msgSend(ptr %msg_result843, ptr %sel846, ptr %msg_result845, double 1.000000e+00, double -3.200000e+01)
  %sel848 = call ptr @sel_registerName(ptr @sel_name.583)
  call void @objc_msgSend(ptr %msg_result847, ptr %sel848, i1 true)
  %sel849 = call ptr @sel_registerName(ptr @sel_name.584)
  %msg_result850 = call ptr @objc_msgSend(ptr %msg_result748, ptr %sel849)
  %sel851 = call ptr @sel_registerName(ptr @sel_name.585)
  %msg_result852 = call ptr @objc_msgSend(ptr %msg_result850, ptr %sel851, double 1.900000e+02)
  %sel853 = call ptr @sel_registerName(ptr @sel_name.586)
  call void @objc_msgSend(ptr %msg_result852, ptr %sel853, i1 true)
  %sel854 = call ptr @sel_registerName(ptr @sel_name.587)
  call void @objc_msgSend(ptr %msg_result118, ptr %sel854, i8 0)
  %sel855 = call ptr @sel_registerName(ptr @sel_name.588)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel855, ptr %msg_result118)
  %sel856 = call ptr @sel_registerName(ptr @sel_name.589)
  %msg_result857 = call ptr @objc_msgSend(ptr %msg_result118, ptr %sel856)
  %sel858 = call ptr @sel_registerName(ptr @sel_name.590)
  %msg_result859 = call ptr @objc_msgSend(ptr %msg_result7, ptr %sel858)
  %sel860 = call ptr @sel_registerName(ptr @sel_name.591)
  %msg_result861 = call ptr @objc_msgSend(ptr %msg_result857, ptr %sel860, ptr %msg_result859, double 1.000000e+00, double -0.000000e+00)
  %sel862 = call ptr @sel_registerName(ptr @sel_name.592)
  call void @objc_msgSend(ptr %msg_result861, ptr %sel862, i1 true)
  %cls863 = call ptr @objc_getClass(ptr @class_name.593)
  %sel864 = call ptr @sel_registerName(ptr @sel_name.594)
  %msg_result865 = call ptr @objc_msgSend(ptr %cls863, ptr %sel864)
  %sel866 = call ptr @sel_registerName(ptr @sel_name.595)
  %msg_result867 = call ptr @objc_msgSend(ptr %msg_result865, ptr %sel866, { { double, double }, { double, double } } { { double, double } { double 1.600000e+01, double 6.200000e+02 }, { double, double } { double 3.180000e+02, double 6.400000e+01 } })
  %cls868 = call ptr @objc_getClass(ptr @class_name.596)
  %sel869 = call ptr @sel_registerName(ptr @sel_name.597)
  %msg_result870 = call ptr @objc_msgSend(ptr %cls868, ptr %sel869)
  %sel871 = call ptr @sel_registerName(ptr @sel_name.598)
  call void @objc_msgSend(ptr %msg_result867, ptr %sel871, ptr %msg_result870)
  %sel872 = call ptr @sel_registerName(ptr @sel_name.599)
  %msg_result873 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel872)
  %sel874 = call ptr @sel_registerName(ptr @sel_name.600)
  call void @objc_msgSend(ptr %msg_result873, ptr %sel874, double 1.000000e+00)
  %cls875 = call ptr @objc_getClass(ptr @class_name.601)
  %sel876 = call ptr @sel_registerName(ptr @sel_name.602)
  %msg_result877 = call ptr @objc_msgSend(ptr %cls875, ptr %sel876, double 0x3FEBDBDBDBDBDBDC, double 0x3FEC9C9C9C9C9C9D, double 0x3FEDDDDDDDDDDDDE, double 1.000000e+00)
  %sel878 = call ptr @sel_registerName(ptr @sel_name.603)
  %msg_result879 = call ptr @objc_msgSend(ptr %msg_result877, ptr %sel878)
  %sel880 = call ptr @sel_registerName(ptr @sel_name.604)
  call void @objc_msgSend(ptr %msg_result873, ptr %sel880, ptr %msg_result879)
  %sel881 = call ptr @sel_registerName(ptr @sel_name.605)
  call void @objc_msgSend(ptr %msg_result873, ptr %sel881, double 1.800000e+01)
  %cls882 = call ptr @objc_getClass(ptr @class_name.606)
  %sel883 = call ptr @sel_registerName(ptr @sel_name.607)
  %msg_result884 = call ptr @objc_msgSend(ptr %cls882, ptr %sel883)
  %sel885 = call ptr @sel_registerName(ptr @sel_name.608)
  %msg_result886 = call ptr @objc_msgSend(ptr %msg_result884, ptr %sel885, { { double, double }, { double, double } } { { double, double } { double 1.000000e+01, double 2.360000e+01 }, { double, double } { double 2.352000e+02, double 1.680000e+01 } })
  %cls887 = call ptr @objc_getClass(ptr @class_name.610)
  %sel888 = call ptr @sel_registerName(ptr @sel_name.611)
  %msg_result889 = call ptr @objc_msgSend(ptr %cls887, ptr %sel888)
  %sel890 = call ptr @sel_registerName(ptr @sel_name.612)
  %msg_result891 = call ptr @objc_msgSend(ptr %msg_result889, ptr %sel890, ptr @text_content.609)
  %sel892 = call ptr @sel_registerName(ptr @sel_name.613)
  call void @objc_msgSend(ptr %msg_result886, ptr %sel892, ptr %msg_result891)
  %cls893 = call ptr @objc_getClass(ptr @class_name.614)
  %sel894 = call ptr @sel_registerName(ptr @sel_name.615)
  %msg_result895 = call ptr @objc_msgSend(ptr %cls893, ptr %sel894)
  %sel896 = call ptr @sel_registerName(ptr @sel_name.616)
  call void @objc_msgSend(ptr %msg_result886, ptr %sel896, ptr %msg_result895)
  %cls897 = call ptr @objc_getClass(ptr @class_name.617)
  %sel898 = call ptr @sel_registerName(ptr @sel_name.618)
  %msg_result899 = call ptr @objc_msgSend(ptr %cls897, ptr %sel898, double 1.400000e+01)
  %sel900 = call ptr @sel_registerName(ptr @sel_name.619)
  call void @objc_msgSend(ptr %msg_result886, ptr %sel900, ptr %msg_result899)
  %sel901 = call ptr @sel_registerName(ptr @sel_name.620)
  call void @objc_msgSend(ptr %msg_result867, ptr %sel901, ptr %msg_result886)
  %cls902 = call ptr @objc_getClass(ptr @class_name.621)
  %sel903 = call ptr @sel_registerName(ptr @sel_name.622)
  %msg_result904 = call ptr @objc_msgSend(ptr %cls902, ptr %sel903, i64 0)
  %sel905 = call ptr @sel_registerName(ptr @sel_name.623)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel905, { { double, double }, { double, double } } { { double, double } { double 2.452000e+02, double 1.000000e+01 }, { double, double } { double 1.000000e+02, double 4.400000e+01 } })
  %cls906 = call ptr @objc_getClass(ptr @class_name.625)
  %sel907 = call ptr @sel_registerName(ptr @sel_name.626)
  %msg_result908 = call ptr @objc_msgSend(ptr %cls906, ptr %sel907)
  %sel909 = call ptr @sel_registerName(ptr @sel_name.627)
  %msg_result910 = call ptr @objc_msgSend(ptr %msg_result908, ptr %sel909, ptr @text_content.624)
  %sel911 = call ptr @sel_registerName(ptr @sel_name.628)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel911, ptr %msg_result910, i64 0)
  %cls912 = call ptr @objc_getClass(ptr @class_name.629)
  %sel913 = call ptr @sel_registerName(ptr @sel_name.630)
  %msg_result914 = call ptr @objc_msgSend(ptr %cls912, ptr %sel913, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel915 = call ptr @sel_registerName(ptr @sel_name.631)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel915, ptr %msg_result914)
  %sel916 = call ptr @sel_registerName(ptr @sel_name.632)
  %msg_result917 = call ptr @objc_msgSend(ptr %msg_result904, ptr %sel916)
  %sel918 = call ptr @sel_registerName(ptr @sel_name.633)
  call void @objc_msgSend(ptr %msg_result917, ptr %sel918, double 1.200000e+01)
  %cls919 = call ptr @objc_getClass(ptr @class_name.634)
  %sel920 = call ptr @sel_registerName(ptr @sel_name.635)
  %msg_result921 = call ptr @objc_msgSend(ptr %cls919, ptr %sel920)
  %sel922 = call ptr @sel_registerName(ptr @sel_name.636)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel922, ptr %msg_result921, i64 0)
  %sel923 = call ptr @sel_registerName(ptr @sel_name.637)
  %sel924 = call ptr @sel_registerName(ptr @sel_name.638)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel924, ptr %0, ptr %sel923, i64 64)
  %sel925 = call ptr @sel_registerName(ptr @sel_name.639)
  call void @objc_msgSend(ptr %msg_result867, ptr %sel925, ptr %msg_result904)
  %sel926 = call ptr @sel_registerName(ptr @sel_name.640)
  call void @objc_msgSend(ptr %msg_result886, ptr %sel926, i1 false)
  %sel927 = call ptr @sel_registerName(ptr @sel_name.641)
  %msg_result928 = call ptr @objc_msgSend(ptr %msg_result886, ptr %sel927)
  %sel929 = call ptr @sel_registerName(ptr @sel_name.642)
  %msg_result930 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel929)
  %sel931 = call ptr @sel_registerName(ptr @sel_name.643)
  %msg_result932 = call ptr @objc_msgSend(ptr %msg_result928, ptr %sel931, ptr %msg_result930, double 0.000000e+00)
  %sel933 = call ptr @sel_registerName(ptr @sel_name.644)
  call void @objc_msgSend(ptr %msg_result932, ptr %sel933, i1 true)
  %sel934 = call ptr @sel_registerName(ptr @sel_name.645)
  %msg_result935 = call ptr @objc_msgSend(ptr %msg_result886, ptr %sel934)
  %sel936 = call ptr @sel_registerName(ptr @sel_name.646)
  %msg_result937 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel936)
  %sel938 = call ptr @sel_registerName(ptr @sel_name.647)
  %msg_result939 = call ptr @objc_msgSend(ptr %msg_result935, ptr %sel938, ptr %msg_result937, double 1.000000e+01)
  %sel940 = call ptr @sel_registerName(ptr @sel_name.648)
  call void @objc_msgSend(ptr %msg_result939, ptr %sel940, i1 true)
  %sel941 = call ptr @sel_registerName(ptr @sel_name.649)
  call void @objc_msgSend(ptr %msg_result904, ptr %sel941, i1 false)
  %sel942 = call ptr @sel_registerName(ptr @sel_name.650)
  %msg_result943 = call ptr @objc_msgSend(ptr %msg_result904, ptr %sel942)
  %sel944 = call ptr @sel_registerName(ptr @sel_name.651)
  %msg_result945 = call ptr @objc_msgSend(ptr %msg_result943, ptr %sel944, double 1.000000e+02)
  %sel946 = call ptr @sel_registerName(ptr @sel_name.652)
  call void @objc_msgSend(ptr %msg_result945, ptr %sel946, i1 true)
  %sel947 = call ptr @sel_registerName(ptr @sel_name.653)
  %msg_result948 = call ptr @objc_msgSend(ptr %msg_result904, ptr %sel947)
  %sel949 = call ptr @sel_registerName(ptr @sel_name.654)
  %msg_result950 = call ptr @objc_msgSend(ptr %msg_result948, ptr %sel949, double 4.400000e+01)
  %sel951 = call ptr @sel_registerName(ptr @sel_name.655)
  call void @objc_msgSend(ptr %msg_result950, ptr %sel951, i1 true)
  %sel952 = call ptr @sel_registerName(ptr @sel_name.656)
  %msg_result953 = call ptr @objc_msgSend(ptr %msg_result904, ptr %sel952)
  %sel954 = call ptr @sel_registerName(ptr @sel_name.657)
  %msg_result955 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel954)
  %sel956 = call ptr @sel_registerName(ptr @sel_name.658)
  %msg_result957 = call ptr @objc_msgSend(ptr %msg_result953, ptr %sel956, ptr %msg_result955, double 0.000000e+00)
  %sel958 = call ptr @sel_registerName(ptr @sel_name.659)
  call void @objc_msgSend(ptr %msg_result957, ptr %sel958, i1 true)
  %sel959 = call ptr @sel_registerName(ptr @sel_name.660)
  %msg_result960 = call ptr @objc_msgSend(ptr %msg_result904, ptr %sel959)
  %sel961 = call ptr @sel_registerName(ptr @sel_name.661)
  %msg_result962 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel961)
  %sel963 = call ptr @sel_registerName(ptr @sel_name.662)
  %msg_result964 = call ptr @objc_msgSend(ptr %msg_result960, ptr %sel963, ptr %msg_result962, double -1.000000e+01)
  %sel965 = call ptr @sel_registerName(ptr @sel_name.663)
  call void @objc_msgSend(ptr %msg_result964, ptr %sel965, i1 true)
  %sel966 = call ptr @sel_registerName(ptr @sel_name.664)
  call void @objc_msgSend(ptr %msg_result7, ptr %sel966, ptr %msg_result867)
  %sel967 = call ptr @sel_registerName(ptr @sel_name.665)
  call void @objc_msgSend(ptr %msg_result867, ptr %sel967, i1 false)
  %sel968 = call ptr @sel_registerName(ptr @sel_name.666)
  %msg_result969 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel968)
  %sel970 = call ptr @sel_registerName(ptr @sel_name.667)
  %msg_result971 = call ptr @objc_msgSend(ptr %msg_result969, ptr %sel970, double 6.400000e+01)
  %sel972 = call ptr @sel_registerName(ptr @sel_name.668)
  call void @objc_msgSend(ptr %msg_result971, ptr %sel972, i1 true)
  %sel973 = call ptr @sel_registerName(ptr @sel_name.669)
  %msg_result974 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel973)
  %sel975 = call ptr @sel_registerName(ptr @sel_name.670)
  %msg_result976 = call ptr @objc_msgSend(ptr %msg_result7, ptr %sel975)
  %sel977 = call ptr @sel_registerName(ptr @sel_name.671)
  %msg_result978 = call ptr @objc_msgSend(ptr %msg_result974, ptr %sel977, ptr %msg_result976, double 1.600000e+01)
  %sel979 = call ptr @sel_registerName(ptr @sel_name.672)
  call void @objc_msgSend(ptr %msg_result978, ptr %sel979, i1 true)
  %sel980 = call ptr @sel_registerName(ptr @sel_name.673)
  %msg_result981 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel980)
  %sel982 = call ptr @sel_registerName(ptr @sel_name.674)
  %msg_result983 = call ptr @objc_msgSend(ptr %msg_result7, ptr %sel982)
  %sel984 = call ptr @sel_registerName(ptr @sel_name.675)
  %msg_result985 = call ptr @objc_msgSend(ptr %msg_result981, ptr %sel984, ptr %msg_result983, double -1.600000e+01)
  %sel986 = call ptr @sel_registerName(ptr @sel_name.676)
  call void @objc_msgSend(ptr %msg_result985, ptr %sel986, i1 true)
  %sel987 = call ptr @sel_registerName(ptr @sel_name.677)
  %msg_result988 = call ptr @objc_msgSend(ptr %msg_result867, ptr %sel987)
  %sel989 = call ptr @sel_registerName(ptr @sel_name.678)
  %msg_result990 = call ptr @objc_msgSend(ptr %msg_result7, ptr %sel989)
  %sel991 = call ptr @sel_registerName(ptr @sel_name.679)
  %msg_result992 = call ptr @objc_msgSend(ptr %msg_result988, ptr %sel991, ptr %msg_result990, double -1.600000e+01)
  %sel993 = call ptr @sel_registerName(ptr @sel_name.680)
  call void @objc_msgSend(ptr %msg_result992, ptr %sel993, i1 true)
  store ptr %msg_result7, ptr @g_root_view, align 8
  %cls994 = call ptr @objc_getClass(ptr @class_name.681)
  %sel995 = call ptr @sel_registerName(ptr @sel_name.682)
  %msg_result996 = call ptr @objc_msgSend(ptr %cls994, ptr %sel995)
  %sel997 = call ptr @sel_registerName(ptr @sel_name.683)
  %msg_result998 = call { { double, double }, { double, double } } @objc_msgSend(ptr %msg_result996, ptr %sel997)
  %cls999 = call ptr @objc_getClass(ptr @class_name.684)
  %sel1000 = call ptr @sel_registerName(ptr @sel_name.685)
  %msg_result1001 = call ptr @objc_msgSend(ptr %cls999, ptr %sel1000)
  %sel1002 = call ptr @sel_registerName(ptr @sel_name.686)
  %msg_result1003 = call ptr @objc_msgSend(ptr %msg_result1001, ptr %sel1002, { { double, double }, { double, double } } %msg_result998)
  %cls1004 = call ptr @objc_getClass(ptr @class_name.687)
  %sel1005 = call ptr @sel_registerName(ptr @sel_name.688)
  %msg_result1006 = call ptr @objc_msgSend(ptr %cls1004, ptr %sel1005)
  %sel1007 = call ptr @sel_registerName(ptr @sel_name.689)
  call void @objc_msgSend(ptr %msg_result1003, ptr %sel1007, ptr %msg_result1006)
  %cls1008 = call ptr @objc_getClass(ptr @class_name.690)
  %sel1009 = call ptr @sel_registerName(ptr @sel_name.691)
  %msg_result1010 = call ptr @objc_msgSend(ptr %cls1008, ptr %sel1009)
  %sel1011 = call ptr @sel_registerName(ptr @sel_name.692)
  %msg_result1012 = call ptr @objc_msgSend(ptr %msg_result1010, ptr %sel1011)
  %loaded_view = load ptr, ptr @g_root_view, align 8
  %sel1013 = call ptr @sel_registerName(ptr @sel_name.693)
  %msg_result1014 = call ptr @objc_msgSend(ptr %msg_result1012, ptr %sel1013)
  %sel1015 = call ptr @sel_registerName(ptr @sel_name.694)
  call void @objc_msgSend(ptr %msg_result1014, ptr %sel1015, ptr %msg_result1006)
  %sel1016 = call ptr @sel_registerName(ptr @sel_name.695)
  call void @objc_msgSend(ptr %msg_result1014, ptr %sel1016, ptr %loaded_view)
  %sel1017 = call ptr @sel_registerName(ptr @sel_name.696)
  call void @objc_msgSend(ptr %loaded_view, ptr %sel1017, i1 false)
  %sel1018 = call ptr @sel_registerName(ptr @sel_name.697)
  %msg_result1019 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1018)
  %sel1020 = call ptr @sel_registerName(ptr @sel_name.698)
  %msg_result1021 = call ptr @objc_msgSend(ptr %msg_result1014, ptr %sel1020)
  %sel1022 = call ptr @sel_registerName(ptr @sel_name.699)
  %msg_result1023 = call ptr @objc_msgSend(ptr %msg_result1019, ptr %sel1022, ptr %msg_result1021)
  %sel1024 = call ptr @sel_registerName(ptr @sel_name.700)
  call void @objc_msgSend(ptr %msg_result1023, ptr %sel1024, i1 true)
  %sel1025 = call ptr @sel_registerName(ptr @sel_name.701)
  %msg_result1026 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1025)
  %sel1027 = call ptr @sel_registerName(ptr @sel_name.702)
  %msg_result1028 = call ptr @objc_msgSend(ptr %msg_result1014, ptr %sel1027)
  %sel1029 = call ptr @sel_registerName(ptr @sel_name.703)
  %msg_result1030 = call ptr @objc_msgSend(ptr %msg_result1026, ptr %sel1029, ptr %msg_result1028)
  %sel1031 = call ptr @sel_registerName(ptr @sel_name.704)
  call void @objc_msgSend(ptr %msg_result1030, ptr %sel1031, i1 true)
  %sel1032 = call ptr @sel_registerName(ptr @sel_name.705)
  %msg_result1033 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1032)
  %sel1034 = call ptr @sel_registerName(ptr @sel_name.706)
  %msg_result1035 = call ptr @objc_msgSend(ptr %msg_result1014, ptr %sel1034)
  %sel1036 = call ptr @sel_registerName(ptr @sel_name.707)
  %msg_result1037 = call ptr @objc_msgSend(ptr %msg_result1033, ptr %sel1036, ptr %msg_result1035)
  %sel1038 = call ptr @sel_registerName(ptr @sel_name.708)
  call void @objc_msgSend(ptr %msg_result1037, ptr %sel1038, i1 true)
  %sel1039 = call ptr @sel_registerName(ptr @sel_name.709)
  %msg_result1040 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1039)
  %sel1041 = call ptr @sel_registerName(ptr @sel_name.710)
  %msg_result1042 = call ptr @objc_msgSend(ptr %msg_result1014, ptr %sel1041)
  %sel1043 = call ptr @sel_registerName(ptr @sel_name.711)
  %msg_result1044 = call ptr @objc_msgSend(ptr %msg_result1040, ptr %sel1043, ptr %msg_result1042)
  %sel1045 = call ptr @sel_registerName(ptr @sel_name.712)
  call void @objc_msgSend(ptr %msg_result1044, ptr %sel1045, i1 true)
  %top_background_target = load ptr, ptr @g_top_background_view, align 8
  %cls1046 = call ptr @objc_getClass(ptr @class_name.713)
  %sel1047 = call ptr @sel_registerName(ptr @sel_name.714)
  %msg_result1048 = call ptr @objc_msgSend(ptr %cls1046, ptr %sel1047)
  %sel1049 = call ptr @sel_registerName(ptr @sel_name.715)
  %msg_result1050 = call ptr @objc_msgSend(ptr %msg_result1048, ptr %sel1049)
  %cls1051 = call ptr @objc_getClass(ptr @class_name.716)
  %sel1052 = call ptr @sel_registerName(ptr @sel_name.717)
  %msg_result1053 = call ptr @objc_msgSend(ptr %cls1051, ptr %sel1052, double 0x3FBC1C1C1C1C1C1C, double 0x3FC3939393939394, double 0x3FCE9E9E9E9E9E9F, double 1.000000e+00)
  %sel1054 = call ptr @sel_registerName(ptr @sel_name.718)
  call void @objc_msgSend(ptr %msg_result1050, ptr %sel1054, ptr %msg_result1053)
  %sel1055 = call ptr @sel_registerName(ptr @sel_name.719)
  call void @objc_msgSend(ptr %msg_result1050, ptr %sel1055, i1 false)
  %sel1056 = call ptr @sel_registerName(ptr @sel_name.720)
  call void @objc_msgSend(ptr %msg_result1050, ptr %sel1056, i1 false)
  %sel1057 = call ptr @sel_registerName(ptr @sel_name.721)
  call void @objc_msgSend(ptr %loaded_view, ptr %sel1057, ptr %msg_result1050, i64 0)
  %sel1058 = call ptr @sel_registerName(ptr @sel_name.722)
  %msg_result1059 = call ptr @objc_msgSend(ptr %msg_result1050, ptr %sel1058)
  %sel1060 = call ptr @sel_registerName(ptr @sel_name.723)
  %msg_result1061 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1060)
  %sel1062 = call ptr @sel_registerName(ptr @sel_name.724)
  %msg_result1063 = call ptr @objc_msgSend(ptr %msg_result1059, ptr %sel1062, ptr %msg_result1061, double 0.000000e+00)
  %sel1064 = call ptr @sel_registerName(ptr @sel_name.725)
  call void @objc_msgSend(ptr %msg_result1063, ptr %sel1064, i1 true)
  %sel1065 = call ptr @sel_registerName(ptr @sel_name.726)
  %msg_result1066 = call ptr @objc_msgSend(ptr %msg_result1050, ptr %sel1065)
  %sel1067 = call ptr @sel_registerName(ptr @sel_name.727)
  %msg_result1068 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1067)
  %sel1069 = call ptr @sel_registerName(ptr @sel_name.728)
  %msg_result1070 = call ptr @objc_msgSend(ptr %msg_result1066, ptr %sel1069, ptr %msg_result1068, double 0.000000e+00)
  %sel1071 = call ptr @sel_registerName(ptr @sel_name.729)
  call void @objc_msgSend(ptr %msg_result1070, ptr %sel1071, i1 true)
  %sel1072 = call ptr @sel_registerName(ptr @sel_name.730)
  %msg_result1073 = call ptr @objc_msgSend(ptr %msg_result1050, ptr %sel1072)
  %sel1074 = call ptr @sel_registerName(ptr @sel_name.731)
  %msg_result1075 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel1074)
  %sel1076 = call ptr @sel_registerName(ptr @sel_name.732)
  %msg_result1077 = call ptr @objc_msgSend(ptr %msg_result1073, ptr %sel1076, ptr %msg_result1075, double 0.000000e+00)
  %sel1078 = call ptr @sel_registerName(ptr @sel_name.733)
  call void @objc_msgSend(ptr %msg_result1077, ptr %sel1078, i1 true)
  %sel1079 = call ptr @sel_registerName(ptr @sel_name.734)
  %msg_result1080 = call ptr @objc_msgSend(ptr %msg_result1050, ptr %sel1079)
  %sel1081 = call ptr @sel_registerName(ptr @sel_name.735)
  %msg_result1082 = call ptr @objc_msgSend(ptr %top_background_target, ptr %sel1081)
  %sel1083 = call ptr @sel_registerName(ptr @sel_name.736)
  %msg_result1084 = call ptr @objc_msgSend(ptr %msg_result1080, ptr %sel1083, ptr %msg_result1082, double 0.000000e+00)
  %sel1085 = call ptr @sel_registerName(ptr @sel_name.737)
  call void @objc_msgSend(ptr %msg_result1084, ptr %sel1085, i1 true)
  %sel1086 = call ptr @sel_registerName(ptr @sel_name.738)
  call void @objc_msgSend(ptr %msg_result1050, ptr %sel1086, double 0.000000e+00)
  store ptr %msg_result1050, ptr @g_scroll_fade_background, align 8
  %sel1087 = call ptr @sel_registerName(ptr @sel_name.739)
  call void @objc_msgSend(ptr %msg_result1003, ptr %sel1087, ptr %msg_result1012)
  %sel1088 = call ptr @sel_registerName(ptr @sel_name.740)
  call void @objc_msgSend(ptr %msg_result1003, ptr %sel1088)
  ret i8 1
}

define void @app_delegate_state_action_0(ptr %0, ptr %1, ptr %2) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name.743)
  %sel = call ptr @sel_registerName(ptr @sel_name.744)
  %msg_result = call ptr @objc_msgSend(ptr %cls, ptr %sel)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.745)
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, ptr @text_content.742)
  store ptr %msg_result2, ptr @g_state_value, align 8
  %state_label_0 = load ptr, ptr @g_state_label_0, align 8
  %sel3 = call ptr @sel_registerName(ptr @sel_name.746)
  call void @objc_msgSend(ptr %state_label_0, ptr %sel3, ptr %msg_result2)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.747)
  %msg_result5 = call i8 @objc_msgSend(ptr %msg_result2, ptr %sel4)
  %conditional_new_state = icmp ne i8 %msg_result5, 0
  %conditional_old_state_0 = load i8, ptr @g_conditional_state_0, align 1
  %conditional_old_bool_0 = icmp ne i8 %conditional_old_state_0, 0
  %conditional_changed_0 = icmp ne i1 %conditional_old_bool_0, %conditional_new_state
  br i1 %conditional_changed_0, label %conditional_update_0, label %conditional_next_0

conditional_update_0:                             ; preds = %entry
  %conditional_true_view_0 = load ptr, ptr @g_conditional_true_0, align 8
  %conditional_false_view_0 = load ptr, ptr @g_conditional_false_0, align 8
  %conditional_old_view = select i1 %conditional_old_bool_0, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_new_view = select i1 %conditional_new_state, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_stack_0 = load ptr, ptr @g_conditional_stack_0, align 8
  %sel6 = call ptr @sel_registerName(ptr @sel_name.748)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel6, ptr %conditional_old_view)
  %sel7 = call ptr @sel_registerName(ptr @sel_name.749)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel7, i8 1)
  %sel8 = call ptr @sel_registerName(ptr @sel_name.750)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel8)
  %conditional_no_new_branch = icmp eq ptr %conditional_new_view, null
  br i1 %conditional_no_new_branch, label %conditional_layout_0, label %conditional_insert_0

conditional_next_0:                               ; preds = %conditional_mount_0_join, %entry
  ret void

conditional_insert_0:                             ; preds = %conditional_update_0
  %sel9 = call ptr @sel_registerName(ptr @sel_name.751)
  call void @objc_msgSend(ptr %conditional_new_view, ptr %sel9, i8 0)
  %conditional_index = select i1 %conditional_new_state, i64 5, i64 5
  %sel10 = call ptr @sel_registerName(ptr @sel_name.752)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel10, ptr %conditional_new_view, i64 %conditional_index)
  br label %conditional_layout_0

conditional_layout_0:                             ; preds = %conditional_insert_0, %conditional_update_0
  %sel11 = call ptr @sel_registerName(ptr @sel_name.753)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel11)
  %conditional_store_state = zext i1 %conditional_new_state to i8
  store i8 %conditional_store_state, ptr @g_conditional_state_0, align 1
  br i1 %conditional_old_bool_0, label %conditional_unmount_0_true, label %conditional_unmount_0_false

conditional_unmount_0_true:                       ; preds = %conditional_layout_0
  %sel12 = call ptr @sel_registerName(ptr @sel_name.754)
  call void @objc_msgSend(ptr %0, ptr %sel12, ptr %0)
  br label %conditional_unmount_0_join

conditional_unmount_0_false:                      ; preds = %conditional_layout_0
  br label %conditional_unmount_0_join

conditional_unmount_0_join:                       ; preds = %conditional_unmount_0_false, %conditional_unmount_0_true
  br i1 %conditional_new_state, label %conditional_mount_0_true, label %conditional_mount_0_false

conditional_mount_0_true:                         ; preds = %conditional_unmount_0_join
  %sel13 = call ptr @sel_registerName(ptr @sel_name.755)
  call void @objc_msgSend(ptr %0, ptr %sel13, ptr %0)
  br label %conditional_mount_0_join

conditional_mount_0_false:                        ; preds = %conditional_unmount_0_join
  br label %conditional_mount_0_join

conditional_mount_0_join:                         ; preds = %conditional_mount_0_false, %conditional_mount_0_true
  br label %conditional_next_0
}

define void @app_delegate_state_action_1(ptr %0, ptr %1, ptr %2) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name.758)
  %sel = call ptr @sel_registerName(ptr @sel_name.759)
  %msg_result = call ptr @objc_msgSend(ptr %cls, ptr %sel)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.760)
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, ptr @text_content.757)
  store ptr %msg_result2, ptr @g_state_value, align 8
  %state_label_0 = load ptr, ptr @g_state_label_0, align 8
  %sel3 = call ptr @sel_registerName(ptr @sel_name.761)
  call void @objc_msgSend(ptr %state_label_0, ptr %sel3, ptr %msg_result2)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.762)
  %msg_result5 = call i8 @objc_msgSend(ptr %msg_result2, ptr %sel4)
  %conditional_new_state = icmp ne i8 %msg_result5, 0
  %conditional_old_state_0 = load i8, ptr @g_conditional_state_0, align 1
  %conditional_old_bool_0 = icmp ne i8 %conditional_old_state_0, 0
  %conditional_changed_0 = icmp ne i1 %conditional_old_bool_0, %conditional_new_state
  br i1 %conditional_changed_0, label %conditional_update_0, label %conditional_next_0

conditional_update_0:                             ; preds = %entry
  %conditional_true_view_0 = load ptr, ptr @g_conditional_true_0, align 8
  %conditional_false_view_0 = load ptr, ptr @g_conditional_false_0, align 8
  %conditional_old_view = select i1 %conditional_old_bool_0, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_new_view = select i1 %conditional_new_state, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_stack_0 = load ptr, ptr @g_conditional_stack_0, align 8
  %sel6 = call ptr @sel_registerName(ptr @sel_name.763)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel6, ptr %conditional_old_view)
  %sel7 = call ptr @sel_registerName(ptr @sel_name.764)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel7, i8 1)
  %sel8 = call ptr @sel_registerName(ptr @sel_name.765)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel8)
  %conditional_no_new_branch = icmp eq ptr %conditional_new_view, null
  br i1 %conditional_no_new_branch, label %conditional_layout_0, label %conditional_insert_0

conditional_next_0:                               ; preds = %conditional_mount_0_join, %entry
  ret void

conditional_insert_0:                             ; preds = %conditional_update_0
  %sel9 = call ptr @sel_registerName(ptr @sel_name.766)
  call void @objc_msgSend(ptr %conditional_new_view, ptr %sel9, i8 0)
  %conditional_index = select i1 %conditional_new_state, i64 5, i64 5
  %sel10 = call ptr @sel_registerName(ptr @sel_name.767)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel10, ptr %conditional_new_view, i64 %conditional_index)
  br label %conditional_layout_0

conditional_layout_0:                             ; preds = %conditional_insert_0, %conditional_update_0
  %sel11 = call ptr @sel_registerName(ptr @sel_name.768)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel11)
  %conditional_store_state = zext i1 %conditional_new_state to i8
  store i8 %conditional_store_state, ptr @g_conditional_state_0, align 1
  br i1 %conditional_old_bool_0, label %conditional_unmount_0_true, label %conditional_unmount_0_false

conditional_unmount_0_true:                       ; preds = %conditional_layout_0
  %sel12 = call ptr @sel_registerName(ptr @sel_name.769)
  call void @objc_msgSend(ptr %0, ptr %sel12, ptr %0)
  br label %conditional_unmount_0_join

conditional_unmount_0_false:                      ; preds = %conditional_layout_0
  br label %conditional_unmount_0_join

conditional_unmount_0_join:                       ; preds = %conditional_unmount_0_false, %conditional_unmount_0_true
  br i1 %conditional_new_state, label %conditional_mount_0_true, label %conditional_mount_0_false

conditional_mount_0_true:                         ; preds = %conditional_unmount_0_join
  %sel13 = call ptr @sel_registerName(ptr @sel_name.770)
  call void @objc_msgSend(ptr %0, ptr %sel13, ptr %0)
  br label %conditional_mount_0_join

conditional_mount_0_false:                        ; preds = %conditional_unmount_0_join
  br label %conditional_mount_0_join

conditional_mount_0_join:                         ; preds = %conditional_mount_0_false, %conditional_mount_0_true
  br label %conditional_next_0
}

define void @app_delegate_state_action_2(ptr %0, ptr %1, ptr %2) {
entry:
  ret void
}

define void @app_delegate_state_action_3(ptr %0, ptr %1, ptr %2) {
entry:
  ret void
}

define void @app_delegate_state_action_4(ptr %0, ptr %1, ptr %2) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name.775)
  %sel = call ptr @sel_registerName(ptr @sel_name.776)
  %msg_result = call ptr @objc_msgSend(ptr %cls, ptr %sel)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.777)
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, ptr @text_content.774)
  store ptr %msg_result2, ptr @g_state_value, align 8
  %state_label_0 = load ptr, ptr @g_state_label_0, align 8
  %sel3 = call ptr @sel_registerName(ptr @sel_name.778)
  call void @objc_msgSend(ptr %state_label_0, ptr %sel3, ptr %msg_result2)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.779)
  %msg_result5 = call i8 @objc_msgSend(ptr %msg_result2, ptr %sel4)
  %conditional_new_state = icmp ne i8 %msg_result5, 0
  %conditional_old_state_0 = load i8, ptr @g_conditional_state_0, align 1
  %conditional_old_bool_0 = icmp ne i8 %conditional_old_state_0, 0
  %conditional_changed_0 = icmp ne i1 %conditional_old_bool_0, %conditional_new_state
  br i1 %conditional_changed_0, label %conditional_update_0, label %conditional_next_0

conditional_update_0:                             ; preds = %entry
  %conditional_true_view_0 = load ptr, ptr @g_conditional_true_0, align 8
  %conditional_false_view_0 = load ptr, ptr @g_conditional_false_0, align 8
  %conditional_old_view = select i1 %conditional_old_bool_0, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_new_view = select i1 %conditional_new_state, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_stack_0 = load ptr, ptr @g_conditional_stack_0, align 8
  %sel6 = call ptr @sel_registerName(ptr @sel_name.780)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel6, ptr %conditional_old_view)
  %sel7 = call ptr @sel_registerName(ptr @sel_name.781)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel7, i8 1)
  %sel8 = call ptr @sel_registerName(ptr @sel_name.782)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel8)
  %conditional_no_new_branch = icmp eq ptr %conditional_new_view, null
  br i1 %conditional_no_new_branch, label %conditional_layout_0, label %conditional_insert_0

conditional_next_0:                               ; preds = %conditional_mount_0_join, %entry
  ret void

conditional_insert_0:                             ; preds = %conditional_update_0
  %sel9 = call ptr @sel_registerName(ptr @sel_name.783)
  call void @objc_msgSend(ptr %conditional_new_view, ptr %sel9, i8 0)
  %conditional_index = select i1 %conditional_new_state, i64 5, i64 5
  %sel10 = call ptr @sel_registerName(ptr @sel_name.784)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel10, ptr %conditional_new_view, i64 %conditional_index)
  br label %conditional_layout_0

conditional_layout_0:                             ; preds = %conditional_insert_0, %conditional_update_0
  %sel11 = call ptr @sel_registerName(ptr @sel_name.785)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel11)
  %conditional_store_state = zext i1 %conditional_new_state to i8
  store i8 %conditional_store_state, ptr @g_conditional_state_0, align 1
  br i1 %conditional_old_bool_0, label %conditional_unmount_0_true, label %conditional_unmount_0_false

conditional_unmount_0_true:                       ; preds = %conditional_layout_0
  %sel12 = call ptr @sel_registerName(ptr @sel_name.786)
  call void @objc_msgSend(ptr %0, ptr %sel12, ptr %0)
  br label %conditional_unmount_0_join

conditional_unmount_0_false:                      ; preds = %conditional_layout_0
  br label %conditional_unmount_0_join

conditional_unmount_0_join:                       ; preds = %conditional_unmount_0_false, %conditional_unmount_0_true
  br i1 %conditional_new_state, label %conditional_mount_0_true, label %conditional_mount_0_false

conditional_mount_0_true:                         ; preds = %conditional_unmount_0_join
  %sel13 = call ptr @sel_registerName(ptr @sel_name.787)
  call void @objc_msgSend(ptr %0, ptr %sel13, ptr %0)
  br label %conditional_mount_0_join

conditional_mount_0_false:                        ; preds = %conditional_unmount_0_join
  br label %conditional_mount_0_join

conditional_mount_0_join:                         ; preds = %conditional_mount_0_false, %conditional_mount_0_true
  br label %conditional_next_0
}

define void @app_delegate_update_state_from_input(ptr %0, ptr %1, ptr %2) {
entry:
  %sel = call ptr @sel_registerName(ptr @sel_name.789)
  %msg_result = call ptr @objc_msgSend(ptr %2, ptr %sel)
  store ptr %msg_result, ptr @g_state_value, align 8
  %state_label_0 = load ptr, ptr @g_state_label_0, align 8
  %sel1 = call ptr @sel_registerName(ptr @sel_name.790)
  call void @objc_msgSend(ptr %state_label_0, ptr %sel1, ptr %msg_result)
  %sel2 = call ptr @sel_registerName(ptr @sel_name.791)
  %msg_result3 = call i8 @objc_msgSend(ptr %msg_result, ptr %sel2)
  %conditional_new_state = icmp ne i8 %msg_result3, 0
  %conditional_old_state_0 = load i8, ptr @g_conditional_state_0, align 1
  %conditional_old_bool_0 = icmp ne i8 %conditional_old_state_0, 0
  %conditional_changed_0 = icmp ne i1 %conditional_old_bool_0, %conditional_new_state
  br i1 %conditional_changed_0, label %conditional_update_0, label %conditional_next_0

conditional_update_0:                             ; preds = %entry
  %conditional_true_view_0 = load ptr, ptr @g_conditional_true_0, align 8
  %conditional_false_view_0 = load ptr, ptr @g_conditional_false_0, align 8
  %conditional_old_view = select i1 %conditional_old_bool_0, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_new_view = select i1 %conditional_new_state, ptr %conditional_true_view_0, ptr %conditional_false_view_0
  %conditional_stack_0 = load ptr, ptr @g_conditional_stack_0, align 8
  %sel4 = call ptr @sel_registerName(ptr @sel_name.792)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel4, ptr %conditional_old_view)
  %sel5 = call ptr @sel_registerName(ptr @sel_name.793)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel5, i8 1)
  %sel6 = call ptr @sel_registerName(ptr @sel_name.794)
  call void @objc_msgSend(ptr %conditional_old_view, ptr %sel6)
  %conditional_no_new_branch = icmp eq ptr %conditional_new_view, null
  br i1 %conditional_no_new_branch, label %conditional_layout_0, label %conditional_insert_0

conditional_next_0:                               ; preds = %conditional_mount_0_join, %entry
  ret void

conditional_insert_0:                             ; preds = %conditional_update_0
  %sel7 = call ptr @sel_registerName(ptr @sel_name.795)
  call void @objc_msgSend(ptr %conditional_new_view, ptr %sel7, i8 0)
  %conditional_index = select i1 %conditional_new_state, i64 5, i64 5
  %sel8 = call ptr @sel_registerName(ptr @sel_name.796)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel8, ptr %conditional_new_view, i64 %conditional_index)
  br label %conditional_layout_0

conditional_layout_0:                             ; preds = %conditional_insert_0, %conditional_update_0
  %sel9 = call ptr @sel_registerName(ptr @sel_name.797)
  call void @objc_msgSend(ptr %conditional_stack_0, ptr %sel9)
  %conditional_store_state = zext i1 %conditional_new_state to i8
  store i8 %conditional_store_state, ptr @g_conditional_state_0, align 1
  br i1 %conditional_old_bool_0, label %conditional_unmount_0_true, label %conditional_unmount_0_false

conditional_unmount_0_true:                       ; preds = %conditional_layout_0
  %sel10 = call ptr @sel_registerName(ptr @sel_name.798)
  call void @objc_msgSend(ptr %0, ptr %sel10, ptr %0)
  br label %conditional_unmount_0_join

conditional_unmount_0_false:                      ; preds = %conditional_layout_0
  br label %conditional_unmount_0_join

conditional_unmount_0_join:                       ; preds = %conditional_unmount_0_false, %conditional_unmount_0_true
  br i1 %conditional_new_state, label %conditional_mount_0_true, label %conditional_mount_0_false

conditional_mount_0_true:                         ; preds = %conditional_unmount_0_join
  %sel11 = call ptr @sel_registerName(ptr @sel_name.799)
  call void @objc_msgSend(ptr %0, ptr %sel11, ptr %0)
  br label %conditional_mount_0_join

conditional_mount_0_false:                        ; preds = %conditional_unmount_0_join
  br label %conditional_mount_0_join

conditional_mount_0_join:                         ; preds = %conditional_mount_0_false, %conditional_mount_0_true
  br label %conditional_next_0
}

define void @app_delegate_scroll_view_did_scroll(ptr %0, ptr %1, ptr %2) {
entry:
  %fade_scroll = load ptr, ptr @g_scroll_fade_scroll, align 8
  %is_fade_scroll = icmp eq ptr %2, %fade_scroll
  br i1 %is_fade_scroll, label %update_fade, label %return

update_fade:                                      ; preds = %entry
  %sel = call ptr @sel_registerName(ptr @sel_name.801)
  %msg_result = call { double, double } @objc_msgSend(ptr %2, ptr %sel)
  %content_offset_y = extractvalue { double, double } %msg_result, 1
  %sel1 = call ptr @sel_registerName(ptr @sel_name.802)
  %msg_result2 = call { double, double, double, double } @objc_msgSend(ptr %2, ptr %sel1)
  %adjusted_inset_top = extractvalue { double, double, double, double } %msg_result2, 0
  %normalized_offset = fadd double %content_offset_y, %adjusted_inset_top
  %offset_after_fade_start = fsub double %normalized_offset, 0.000000e+00
  %past_fade_start = fcmp ogt double %offset_after_fade_start, 0.000000e+00
  %clamped_fade_offset = select i1 %past_fade_start, double %offset_after_fade_start, double 0.000000e+00
  %fade_fraction = fdiv double %clamped_fade_offset, 1.200000e+02
  %before_fade_end = fcmp olt double %fade_fraction, 1.000000e+00
  %clamped_fade_alpha = select i1 %before_fade_end, double %fade_fraction, double 1.000000e+00
  %fade_target = load ptr, ptr @g_scroll_fade_view, align 8
  %sel3 = call ptr @sel_registerName(ptr @sel_name.803)
  call void @objc_msgSend(ptr %fade_target, ptr %sel3, double %clamped_fade_alpha)
  %fade_background = load ptr, ptr @g_scroll_fade_background, align 8
  %sel4 = call ptr @sel_registerName(ptr @sel_name.804)
  call void @objc_msgSend(ptr %fade_background, ptr %sel4, double %clamped_fade_alpha)
  br label %return

return:                                           ; preds = %update_fade, %entry
  ret void
}

define void @app_delegate_focus_event(ptr %0, ptr %1, ptr %2) {
entry:
  ret void
}

declare i32 @UIApplicationMain(i32, ptr, ptr, ptr)
