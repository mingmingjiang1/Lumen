; ModuleID = 'ios_app'
source_filename = "ios_app"
target triple = "arm64-apple-ios"

@g_root_view = global ptr null
@g_state_value = global i64 0
@g_state_label = global ptr null
@class_name = private unnamed_addr constant [9 x i8] c"NSObject\00", align 1
@app_delegate_class_name = private unnamed_addr constant [12 x i8] c"AppDelegate\00", align 1
@class_name.1 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.2 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.3 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.4 = private unnamed_addr constant [10 x i8] c"blueColor\00", align 1
@sel_name.5 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.6 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.7 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.8 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.9 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.10 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.11 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.12 = private unnamed_addr constant [6 x i8] c"layer\00", align 1
@sel_name.13 = private unnamed_addr constant [16 x i8] c"setBorderWidth:\00", align 1
@class_name.14 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.15 = private unnamed_addr constant [31 x i8] c"colorWithRed:green:blue:alpha:\00", align 1
@sel_name.16 = private unnamed_addr constant [8 x i8] c"CGColor\00", align 1
@sel_name.17 = private unnamed_addr constant [16 x i8] c"setBorderColor:\00", align 1
@sel_name.18 = private unnamed_addr constant [17 x i8] c"setCornerRadius:\00", align 1
@sel_name.19 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.20 = private unnamed_addr constant [13 x i8] c"UIScrollView\00", align 1
@sel_name.21 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.22 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.23 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.24 = private unnamed_addr constant [9 x i8] c"redColor\00", align 1
@sel_name.25 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.26 = private unnamed_addr constant [7 x i8] c"UIView\00", align 1
@sel_name.27 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.28 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.29 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.30 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.31 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.32 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.33 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.34 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content = private unnamed_addr constant [2 x i8] c"0\00", align 1
@class_name.35 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.36 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.37 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.38 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.39 = private unnamed_addr constant [18 x i8] c"setTextAlignment:\00", align 1
@class_name.40 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.41 = private unnamed_addr constant [10 x i8] c"blueColor\00", align 1
@sel_name.42 = private unnamed_addr constant [14 x i8] c"setTextColor:\00", align 1
@class_name.43 = private unnamed_addr constant [7 x i8] c"UIFont\00", align 1
@sel_name.44 = private unnamed_addr constant [18 x i8] c"systemFontOfSize:\00", align 1
@sel_name.45 = private unnamed_addr constant [9 x i8] c"setFont:\00", align 1
@sel_name.46 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.47 = private unnamed_addr constant [9 x i8] c"UIButton\00", align 1
@sel_name.48 = private unnamed_addr constant [16 x i8] c"buttonWithType:\00", align 1
@sel_name.49 = private unnamed_addr constant [10 x i8] c"setFrame:\00", align 1
@text_content.50 = private unnamed_addr constant [10 x i8] c"Increment\00", align 1
@class_name.51 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.52 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.53 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.54 = private unnamed_addr constant [19 x i8] c"setTitle:forState:\00", align 1
@class_name.55 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.56 = private unnamed_addr constant [10 x i8] c"blueColor\00", align 1
@sel_name.57 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.58 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.59 = private unnamed_addr constant [9 x i8] c"redColor\00", align 1
@sel_name.60 = private unnamed_addr constant [24 x i8] c"setTitleColor:forState:\00", align 1
@sel_name.61 = private unnamed_addr constant [16 x i8] c"incrementState:\00", align 1
@sel_name.62 = private unnamed_addr constant [35 x i8] c"addTarget:action:forControlEvents:\00", align 1
@sel_name.63 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@text_content.64 = private unnamed_addr constant [5 x i8] c"logo\00", align 1
@class_name.65 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.66 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.67 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@class_name.68 = private unnamed_addr constant [8 x i8] c"UIImage\00", align 1
@sel_name.69 = private unnamed_addr constant [12 x i8] c"imageNamed:\00", align 1
@class_name.70 = private unnamed_addr constant [12 x i8] c"UIImageView\00", align 1
@sel_name.71 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.72 = private unnamed_addr constant [15 x i8] c"initWithImage:\00", align 1
@sel_name.73 = private unnamed_addr constant [10 x i8] c"setFrame:\00", align 1
@sel_name.74 = private unnamed_addr constant [16 x i8] c"setContentMode:\00", align 1
@sel_name.75 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.76 = private unnamed_addr constant [12 x i8] c"UITextField\00", align 1
@sel_name.77 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.78 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.79 = private unnamed_addr constant [11 x i8] c"Enter text\00", align 1
@class_name.80 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.81 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.82 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.83 = private unnamed_addr constant [16 x i8] c"setPlaceholder:\00", align 1
@sel_name.84 = private unnamed_addr constant [16 x i8] c"setBorderStyle:\00", align 1
@sel_name.85 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.86 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.87 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.88 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.89 = private unnamed_addr constant [19 x i8] c"Scrollable content\00", align 1
@class_name.90 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.91 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.92 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.93 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.94 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.95 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.96 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.97 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.98 = private unnamed_addr constant [13 x i8] c"More content\00", align 1
@class_name.99 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.100 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.101 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.102 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.103 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@class_name.104 = private unnamed_addr constant [8 x i8] c"UILabel\00", align 1
@sel_name.105 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.106 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@text_content.107 = private unnamed_addr constant [19 x i8] c"Additional content\00", align 1
@class_name.108 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.109 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.110 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.111 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.112 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.113 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.114 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.115 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.116 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.117 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.118 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.119 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.120 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.121 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.122 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.123 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.124 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.125 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.126 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.127 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.128 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.129 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.130 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.131 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.132 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.133 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.134 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.135 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.136 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.137 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.138 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.139 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.140 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.141 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.142 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.143 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.144 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.145 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.146 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.147 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.148 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.149 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.150 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.151 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.152 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.153 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.154 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.155 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.156 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.157 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.158 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.159 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.160 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.161 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.162 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.163 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.164 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.165 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.166 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.167 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.168 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.169 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.170 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.171 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.172 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.173 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.174 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.175 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.176 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.177 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.178 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.179 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.180 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.181 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.182 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.183 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.184 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.185 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.186 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.187 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.188 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.189 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.190 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.191 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.192 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.193 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.194 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.195 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.196 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.197 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.198 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.199 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.200 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.201 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.202 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.203 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.204 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.205 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.206 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.207 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.208 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.209 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.210 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.211 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.212 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.213 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.214 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.215 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.216 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.217 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.218 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.219 = private unnamed_addr constant [16 x i8] c"setContentSize:\00", align 1
@sel_name.220 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.221 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.222 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.223 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.224 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.225 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.226 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.227 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.228 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.229 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.230 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.231 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.232 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.233 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.234 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.235 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.236 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.237 = private unnamed_addr constant [13 x i8] c"heightAnchor\00", align 1
@sel_name.238 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.239 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.240 = private unnamed_addr constant [12 x i8] c"widthAnchor\00", align 1
@sel_name.241 = private unnamed_addr constant [27 x i8] c"constraintEqualToConstant:\00", align 1
@sel_name.242 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.243 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.244 = private unnamed_addr constant [14 x i8] c"centerXAnchor\00", align 1
@sel_name.245 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.246 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.247 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.248 = private unnamed_addr constant [14 x i8] c"centerYAnchor\00", align 1
@sel_name.249 = private unnamed_addr constant [34 x i8] c"constraintEqualToAnchor:constant:\00", align 1
@sel_name.250 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@class_name.251 = private unnamed_addr constant [9 x i8] c"UIScreen\00", align 1
@sel_name.252 = private unnamed_addr constant [11 x i8] c"mainScreen\00", align 1
@sel_name.253 = private unnamed_addr constant [7 x i8] c"bounds\00", align 1
@class_name.254 = private unnamed_addr constant [9 x i8] c"UIWindow\00", align 1
@sel_name.255 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.256 = private unnamed_addr constant [15 x i8] c"initWithFrame:\00", align 1
@class_name.257 = private unnamed_addr constant [8 x i8] c"UIColor\00", align 1
@sel_name.258 = private unnamed_addr constant [11 x i8] c"whiteColor\00", align 1
@sel_name.259 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@class_name.260 = private unnamed_addr constant [17 x i8] c"UIViewController\00", align 1
@sel_name.261 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.262 = private unnamed_addr constant [5 x i8] c"init\00", align 1
@sel_name.263 = private unnamed_addr constant [5 x i8] c"view\00", align 1
@sel_name.264 = private unnamed_addr constant [20 x i8] c"setBackgroundColor:\00", align 1
@sel_name.265 = private unnamed_addr constant [12 x i8] c"addSubview:\00", align 1
@sel_name.266 = private unnamed_addr constant [46 x i8] c"setTranslatesAutoresizingMaskIntoConstraints:\00", align 1
@sel_name.267 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.268 = private unnamed_addr constant [14 x i8] c"leadingAnchor\00", align 1
@sel_name.269 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.270 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.271 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.272 = private unnamed_addr constant [10 x i8] c"topAnchor\00", align 1
@sel_name.273 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.274 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.275 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.276 = private unnamed_addr constant [15 x i8] c"trailingAnchor\00", align 1
@sel_name.277 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.278 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.279 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.280 = private unnamed_addr constant [13 x i8] c"bottomAnchor\00", align 1
@sel_name.281 = private unnamed_addr constant [25 x i8] c"constraintEqualToAnchor:\00", align 1
@sel_name.282 = private unnamed_addr constant [11 x i8] c"setActive:\00", align 1
@sel_name.283 = private unnamed_addr constant [23 x i8] c"setRootViewController:\00", align 1
@sel_name.284 = private unnamed_addr constant [18 x i8] c"makeKeyAndVisible\00", align 1
@sel_name.285 = private unnamed_addr constant [43 x i8] c"application:didFinishLaunchingWithOptions:\00", align 1
@method_type_encoding = private unnamed_addr constant [6 x i8] c"c@:@@\00", align 1
@g_state_text_buffer = global [32 x i8] zeroinitializer
@state_format = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@class_name.286 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.287 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.288 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1
@sel_name.289 = private unnamed_addr constant [9 x i8] c"setText:\00", align 1
@sel_name.290 = private unnamed_addr constant [16 x i8] c"incrementState:\00", align 1
@state_callback_encoding = private unnamed_addr constant [5 x i8] c"v@:@\00", align 1
@text_content.291 = private unnamed_addr constant [12 x i8] c"AppDelegate\00", align 1
@class_name.292 = private unnamed_addr constant [9 x i8] c"NSString\00", align 1
@sel_name.293 = private unnamed_addr constant [6 x i8] c"alloc\00", align 1
@sel_name.294 = private unnamed_addr constant [20 x i8] c"initWithUTF8String:\00", align 1

declare ptr @objc_getClass(ptr)

declare ptr @sel_registerName(ptr)

declare ptr @objc_msgSend(ptr, ptr, ...)

define i32 @main(i32 %0, ptr %1) {
entry:
  %cls = call ptr @objc_getClass(ptr @class_name)
  %app_delegate_class = call ptr @objc_allocateClassPair(ptr %cls, ptr @app_delegate_class_name, i64 0)
  %sel = call ptr @sel_registerName(ptr @sel_name.285)
  %2 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel, ptr @app_delegate_did_finish_launching, ptr @method_type_encoding)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.290)
  %3 = call i1 @class_addMethod(ptr %app_delegate_class, ptr %sel1, ptr @app_delegate_increment_state, ptr @state_callback_encoding)
  call void @objc_registerClassPair(ptr %app_delegate_class)
  %cls2 = call ptr @objc_getClass(ptr @class_name.292)
  %sel3 = call ptr @sel_registerName(ptr @sel_name.293)
  %msg_result = call ptr @objc_msgSend(ptr %cls2, ptr %sel3)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.294)
  %msg_result5 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel4, ptr @text_content.291)
  %4 = call i32 @UIApplicationMain(i32 %0, ptr %1, ptr null, ptr %msg_result5)
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
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, { { double, double }, { double, double } } { { double, double } zeroinitializer, { double, double } { double 3.500000e+02, double 7.000000e+02 } })
  %cls3 = call ptr @objc_getClass(ptr @class_name.3)
  %sel4 = call ptr @sel_registerName(ptr @sel_name.4)
  %msg_result5 = call ptr @objc_msgSend(ptr %cls3, ptr %sel4)
  %sel6 = call ptr @sel_registerName(ptr @sel_name.5)
  call void @objc_msgSend(ptr %msg_result2, ptr %sel6, ptr %msg_result5)
  %cls7 = call ptr @objc_getClass(ptr @class_name.6)
  %sel8 = call ptr @sel_registerName(ptr @sel_name.7)
  %msg_result9 = call ptr @objc_msgSend(ptr %cls7, ptr %sel8)
  %sel10 = call ptr @sel_registerName(ptr @sel_name.8)
  %msg_result11 = call ptr @objc_msgSend(ptr %msg_result9, ptr %sel10, { { double, double }, { double, double } } { { double, double } { double 7.500000e+01, double 8.300000e+01 }, { double, double } { double 2.000000e+02, double 2.000000e+02 } })
  %cls12 = call ptr @objc_getClass(ptr @class_name.9)
  %sel13 = call ptr @sel_registerName(ptr @sel_name.10)
  %msg_result14 = call ptr @objc_msgSend(ptr %cls12, ptr %sel13, double 0x3FEA7A7A7A7A7A7A, double 0x3FEE5E5E5E5E5E5E, double 0x3FD9191919191919, double 1.000000e+00)
  %sel15 = call ptr @sel_registerName(ptr @sel_name.11)
  call void @objc_msgSend(ptr %msg_result11, ptr %sel15, ptr %msg_result14)
  %sel16 = call ptr @sel_registerName(ptr @sel_name.12)
  %msg_result17 = call ptr @objc_msgSend(ptr %msg_result11, ptr %sel16)
  %sel18 = call ptr @sel_registerName(ptr @sel_name.13)
  call void @objc_msgSend(ptr %msg_result17, ptr %sel18, double 2.000000e+00)
  %cls19 = call ptr @objc_getClass(ptr @class_name.14)
  %sel20 = call ptr @sel_registerName(ptr @sel_name.15)
  %msg_result21 = call ptr @objc_msgSend(ptr %cls19, ptr %sel20, double 1.000000e+00, double 0.000000e+00, double 0.000000e+00, double 5.000000e-01)
  %sel22 = call ptr @sel_registerName(ptr @sel_name.16)
  %msg_result23 = call ptr @objc_msgSend(ptr %msg_result21, ptr %sel22)
  %sel24 = call ptr @sel_registerName(ptr @sel_name.17)
  call void @objc_msgSend(ptr %msg_result17, ptr %sel24, ptr %msg_result23)
  %sel25 = call ptr @sel_registerName(ptr @sel_name.18)
  call void @objc_msgSend(ptr %msg_result17, ptr %sel25, double 1.200000e+01)
  %sel26 = call ptr @sel_registerName(ptr @sel_name.19)
  call void @objc_msgSend(ptr %msg_result2, ptr %sel26, ptr %msg_result11)
  %cls27 = call ptr @objc_getClass(ptr @class_name.20)
  %sel28 = call ptr @sel_registerName(ptr @sel_name.21)
  %msg_result29 = call ptr @objc_msgSend(ptr %cls27, ptr %sel28)
  %sel30 = call ptr @sel_registerName(ptr @sel_name.22)
  %msg_result31 = call ptr @objc_msgSend(ptr %msg_result29, ptr %sel30, { { double, double }, { double, double } } { { double, double } { double 5.000000e+00, double 4.570000e+02 }, { double, double } { double 3.400000e+02, double 1.600000e+02 } })
  %cls32 = call ptr @objc_getClass(ptr @class_name.23)
  %sel33 = call ptr @sel_registerName(ptr @sel_name.24)
  %msg_result34 = call ptr @objc_msgSend(ptr %cls32, ptr %sel33)
  %sel35 = call ptr @sel_registerName(ptr @sel_name.25)
  call void @objc_msgSend(ptr %msg_result31, ptr %sel35, ptr %msg_result34)
  %cls36 = call ptr @objc_getClass(ptr @class_name.26)
  %sel37 = call ptr @sel_registerName(ptr @sel_name.27)
  %msg_result38 = call ptr @objc_msgSend(ptr %cls36, ptr %sel37)
  %sel39 = call ptr @sel_registerName(ptr @sel_name.28)
  %msg_result40 = call ptr @objc_msgSend(ptr %msg_result38, ptr %sel39, { { double, double }, { double, double } } { { double, double } zeroinitializer, { double, double } { double 3.400000e+02, double 4.800000e+02 } })
  %cls41 = call ptr @objc_getClass(ptr @class_name.29)
  %sel42 = call ptr @sel_registerName(ptr @sel_name.30)
  %msg_result43 = call ptr @objc_msgSend(ptr %cls41, ptr %sel42)
  %sel44 = call ptr @sel_registerName(ptr @sel_name.31)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel44, ptr %msg_result43)
  %cls45 = call ptr @objc_getClass(ptr @class_name.32)
  %sel46 = call ptr @sel_registerName(ptr @sel_name.33)
  %msg_result47 = call ptr @objc_msgSend(ptr %cls45, ptr %sel46)
  %sel48 = call ptr @sel_registerName(ptr @sel_name.34)
  %msg_result49 = call ptr @objc_msgSend(ptr %msg_result47, ptr %sel48, { { double, double }, { double, double } } { { double, double } { double 1.450000e+02, double 2.000000e+01 }, { double, double } { double 5.000000e+01, double 5.000000e+01 } })
  %cls50 = call ptr @objc_getClass(ptr @class_name.35)
  %sel51 = call ptr @sel_registerName(ptr @sel_name.36)
  %msg_result52 = call ptr @objc_msgSend(ptr %cls50, ptr %sel51)
  %sel53 = call ptr @sel_registerName(ptr @sel_name.37)
  %msg_result54 = call ptr @objc_msgSend(ptr %msg_result52, ptr %sel53, ptr @text_content)
  %sel55 = call ptr @sel_registerName(ptr @sel_name.38)
  call void @objc_msgSend(ptr %msg_result49, ptr %sel55, ptr %msg_result54)
  %sel56 = call ptr @sel_registerName(ptr @sel_name.39)
  call void @objc_msgSend(ptr %msg_result49, ptr %sel56, i64 1)
  %cls57 = call ptr @objc_getClass(ptr @class_name.40)
  %sel58 = call ptr @sel_registerName(ptr @sel_name.41)
  %msg_result59 = call ptr @objc_msgSend(ptr %cls57, ptr %sel58)
  %sel60 = call ptr @sel_registerName(ptr @sel_name.42)
  call void @objc_msgSend(ptr %msg_result49, ptr %sel60, ptr %msg_result59)
  %cls61 = call ptr @objc_getClass(ptr @class_name.43)
  %sel62 = call ptr @sel_registerName(ptr @sel_name.44)
  %msg_result63 = call ptr @objc_msgSend(ptr %cls61, ptr %sel62, double 2.400000e+01)
  %sel64 = call ptr @sel_registerName(ptr @sel_name.45)
  call void @objc_msgSend(ptr %msg_result49, ptr %sel64, ptr %msg_result63)
  store ptr %msg_result49, ptr @g_state_label, align 8
  %sel65 = call ptr @sel_registerName(ptr @sel_name.46)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel65, ptr %msg_result49)
  %cls66 = call ptr @objc_getClass(ptr @class_name.47)
  %sel67 = call ptr @sel_registerName(ptr @sel_name.48)
  %msg_result68 = call ptr @objc_msgSend(ptr %cls66, ptr %sel67, i64 0)
  %sel69 = call ptr @sel_registerName(ptr @sel_name.49)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel69, { { double, double }, { double, double } } { { double, double } { double 7.000000e+01, double 8.200000e+01 }, { double, double } { double 2.000000e+02, double 4.800000e+01 } })
  %cls70 = call ptr @objc_getClass(ptr @class_name.51)
  %sel71 = call ptr @sel_registerName(ptr @sel_name.52)
  %msg_result72 = call ptr @objc_msgSend(ptr %cls70, ptr %sel71)
  %sel73 = call ptr @sel_registerName(ptr @sel_name.53)
  %msg_result74 = call ptr @objc_msgSend(ptr %msg_result72, ptr %sel73, ptr @text_content.50)
  %sel75 = call ptr @sel_registerName(ptr @sel_name.54)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel75, ptr %msg_result74, i64 0)
  %cls76 = call ptr @objc_getClass(ptr @class_name.55)
  %sel77 = call ptr @sel_registerName(ptr @sel_name.56)
  %msg_result78 = call ptr @objc_msgSend(ptr %cls76, ptr %sel77)
  %sel79 = call ptr @sel_registerName(ptr @sel_name.57)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel79, ptr %msg_result78)
  %cls80 = call ptr @objc_getClass(ptr @class_name.58)
  %sel81 = call ptr @sel_registerName(ptr @sel_name.59)
  %msg_result82 = call ptr @objc_msgSend(ptr %cls80, ptr %sel81)
  %sel83 = call ptr @sel_registerName(ptr @sel_name.60)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel83, ptr %msg_result82, i64 0)
  %sel84 = call ptr @sel_registerName(ptr @sel_name.61)
  %sel85 = call ptr @sel_registerName(ptr @sel_name.62)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel85, ptr %0, ptr %sel84, i64 64)
  %sel86 = call ptr @sel_registerName(ptr @sel_name.63)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel86, ptr %msg_result68)
  %cls87 = call ptr @objc_getClass(ptr @class_name.65)
  %sel88 = call ptr @sel_registerName(ptr @sel_name.66)
  %msg_result89 = call ptr @objc_msgSend(ptr %cls87, ptr %sel88)
  %sel90 = call ptr @sel_registerName(ptr @sel_name.67)
  %msg_result91 = call ptr @objc_msgSend(ptr %msg_result89, ptr %sel90, ptr @text_content.64)
  %cls92 = call ptr @objc_getClass(ptr @class_name.68)
  %sel93 = call ptr @sel_registerName(ptr @sel_name.69)
  %msg_result94 = call ptr @objc_msgSend(ptr %cls92, ptr %sel93, ptr %msg_result91)
  %cls95 = call ptr @objc_getClass(ptr @class_name.70)
  %sel96 = call ptr @sel_registerName(ptr @sel_name.71)
  %msg_result97 = call ptr @objc_msgSend(ptr %cls95, ptr %sel96)
  %sel98 = call ptr @sel_registerName(ptr @sel_name.72)
  %msg_result99 = call ptr @objc_msgSend(ptr %msg_result97, ptr %sel98, ptr %msg_result94)
  %sel100 = call ptr @sel_registerName(ptr @sel_name.73)
  call void @objc_msgSend(ptr %msg_result99, ptr %sel100, { { double, double }, { double, double } } { { double, double } { double 7.000000e+01, double 1.420000e+02 }, { double, double } { double 2.000000e+02, double 1.200000e+02 } })
  %sel101 = call ptr @sel_registerName(ptr @sel_name.74)
  call void @objc_msgSend(ptr %msg_result99, ptr %sel101, i64 0)
  %sel102 = call ptr @sel_registerName(ptr @sel_name.75)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel102, ptr %msg_result99)
  %cls103 = call ptr @objc_getClass(ptr @class_name.76)
  %sel104 = call ptr @sel_registerName(ptr @sel_name.77)
  %msg_result105 = call ptr @objc_msgSend(ptr %cls103, ptr %sel104)
  %sel106 = call ptr @sel_registerName(ptr @sel_name.78)
  %msg_result107 = call ptr @objc_msgSend(ptr %msg_result105, ptr %sel106, { { double, double }, { double, double } } { { double, double } { double 2.000000e+01, double 2.620000e+02 }, { double, double } { double 3.000000e+02, double 4.800000e+01 } })
  %cls108 = call ptr @objc_getClass(ptr @class_name.80)
  %sel109 = call ptr @sel_registerName(ptr @sel_name.81)
  %msg_result110 = call ptr @objc_msgSend(ptr %cls108, ptr %sel109)
  %sel111 = call ptr @sel_registerName(ptr @sel_name.82)
  %msg_result112 = call ptr @objc_msgSend(ptr %msg_result110, ptr %sel111, ptr @text_content.79)
  %sel113 = call ptr @sel_registerName(ptr @sel_name.83)
  call void @objc_msgSend(ptr %msg_result107, ptr %sel113, ptr %msg_result112)
  %sel114 = call ptr @sel_registerName(ptr @sel_name.84)
  call void @objc_msgSend(ptr %msg_result107, ptr %sel114, i64 3)
  %sel115 = call ptr @sel_registerName(ptr @sel_name.85)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel115, ptr %msg_result107)
  %cls116 = call ptr @objc_getClass(ptr @class_name.86)
  %sel117 = call ptr @sel_registerName(ptr @sel_name.87)
  %msg_result118 = call ptr @objc_msgSend(ptr %cls116, ptr %sel117)
  %sel119 = call ptr @sel_registerName(ptr @sel_name.88)
  %msg_result120 = call ptr @objc_msgSend(ptr %msg_result118, ptr %sel119, { { double, double }, { double, double } } { { double, double } { double 7.820000e+01, double 3.100000e+02 }, { double, double } { double 1.836000e+02, double 5.000000e+01 } })
  %cls121 = call ptr @objc_getClass(ptr @class_name.90)
  %sel122 = call ptr @sel_registerName(ptr @sel_name.91)
  %msg_result123 = call ptr @objc_msgSend(ptr %cls121, ptr %sel122)
  %sel124 = call ptr @sel_registerName(ptr @sel_name.92)
  %msg_result125 = call ptr @objc_msgSend(ptr %msg_result123, ptr %sel124, ptr @text_content.89)
  %sel126 = call ptr @sel_registerName(ptr @sel_name.93)
  call void @objc_msgSend(ptr %msg_result120, ptr %sel126, ptr %msg_result125)
  %sel127 = call ptr @sel_registerName(ptr @sel_name.94)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel127, ptr %msg_result120)
  %cls128 = call ptr @objc_getClass(ptr @class_name.95)
  %sel129 = call ptr @sel_registerName(ptr @sel_name.96)
  %msg_result130 = call ptr @objc_msgSend(ptr %cls128, ptr %sel129)
  %sel131 = call ptr @sel_registerName(ptr @sel_name.97)
  %msg_result132 = call ptr @objc_msgSend(ptr %msg_result130, ptr %sel131, { { double, double }, { double, double } } { { double, double } { double 0x405B333333333334, double 3.600000e+02 }, { double, double } { double 0x405E999999999999, double 5.000000e+01 } })
  %cls133 = call ptr @objc_getClass(ptr @class_name.99)
  %sel134 = call ptr @sel_registerName(ptr @sel_name.100)
  %msg_result135 = call ptr @objc_msgSend(ptr %cls133, ptr %sel134)
  %sel136 = call ptr @sel_registerName(ptr @sel_name.101)
  %msg_result137 = call ptr @objc_msgSend(ptr %msg_result135, ptr %sel136, ptr @text_content.98)
  %sel138 = call ptr @sel_registerName(ptr @sel_name.102)
  call void @objc_msgSend(ptr %msg_result132, ptr %sel138, ptr %msg_result137)
  %sel139 = call ptr @sel_registerName(ptr @sel_name.103)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel139, ptr %msg_result132)
  %cls140 = call ptr @objc_getClass(ptr @class_name.104)
  %sel141 = call ptr @sel_registerName(ptr @sel_name.105)
  %msg_result142 = call ptr @objc_msgSend(ptr %cls140, ptr %sel141)
  %sel143 = call ptr @sel_registerName(ptr @sel_name.106)
  %msg_result144 = call ptr @objc_msgSend(ptr %msg_result142, ptr %sel143, { { double, double }, { double, double } } { { double, double } { double 7.820000e+01, double 4.100000e+02 }, { double, double } { double 1.836000e+02, double 5.000000e+01 } })
  %cls145 = call ptr @objc_getClass(ptr @class_name.108)
  %sel146 = call ptr @sel_registerName(ptr @sel_name.109)
  %msg_result147 = call ptr @objc_msgSend(ptr %cls145, ptr %sel146)
  %sel148 = call ptr @sel_registerName(ptr @sel_name.110)
  %msg_result149 = call ptr @objc_msgSend(ptr %msg_result147, ptr %sel148, ptr @text_content.107)
  %sel150 = call ptr @sel_registerName(ptr @sel_name.111)
  call void @objc_msgSend(ptr %msg_result144, ptr %sel150, ptr %msg_result149)
  %sel151 = call ptr @sel_registerName(ptr @sel_name.112)
  call void @objc_msgSend(ptr %msg_result40, ptr %sel151, ptr %msg_result144)
  %sel152 = call ptr @sel_registerName(ptr @sel_name.113)
  call void @objc_msgSend(ptr %msg_result49, ptr %sel152, i1 false)
  %sel153 = call ptr @sel_registerName(ptr @sel_name.114)
  %msg_result154 = call ptr @objc_msgSend(ptr %msg_result49, ptr %sel153)
  %sel155 = call ptr @sel_registerName(ptr @sel_name.115)
  %msg_result156 = call ptr @objc_msgSend(ptr %msg_result154, ptr %sel155, double 5.000000e+01)
  %sel157 = call ptr @sel_registerName(ptr @sel_name.116)
  call void @objc_msgSend(ptr %msg_result156, ptr %sel157, i1 true)
  %sel158 = call ptr @sel_registerName(ptr @sel_name.117)
  %msg_result159 = call ptr @objc_msgSend(ptr %msg_result49, ptr %sel158)
  %sel160 = call ptr @sel_registerName(ptr @sel_name.118)
  %msg_result161 = call ptr @objc_msgSend(ptr %msg_result159, ptr %sel160, double 5.000000e+01)
  %sel162 = call ptr @sel_registerName(ptr @sel_name.119)
  call void @objc_msgSend(ptr %msg_result161, ptr %sel162, i1 true)
  %sel163 = call ptr @sel_registerName(ptr @sel_name.120)
  %msg_result164 = call ptr @objc_msgSend(ptr %msg_result49, ptr %sel163)
  %sel165 = call ptr @sel_registerName(ptr @sel_name.121)
  %msg_result166 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel165)
  %sel167 = call ptr @sel_registerName(ptr @sel_name.122)
  %msg_result168 = call ptr @objc_msgSend(ptr %msg_result164, ptr %sel167, ptr %msg_result166, double 0.000000e+00)
  %sel169 = call ptr @sel_registerName(ptr @sel_name.123)
  call void @objc_msgSend(ptr %msg_result168, ptr %sel169, i1 true)
  %sel170 = call ptr @sel_registerName(ptr @sel_name.124)
  %msg_result171 = call ptr @objc_msgSend(ptr %msg_result49, ptr %sel170)
  %sel172 = call ptr @sel_registerName(ptr @sel_name.125)
  %msg_result173 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel172)
  %sel174 = call ptr @sel_registerName(ptr @sel_name.126)
  %msg_result175 = call ptr @objc_msgSend(ptr %msg_result171, ptr %sel174, ptr %msg_result173, double 2.000000e+01)
  %sel176 = call ptr @sel_registerName(ptr @sel_name.127)
  call void @objc_msgSend(ptr %msg_result175, ptr %sel176, i1 true)
  %sel177 = call ptr @sel_registerName(ptr @sel_name.128)
  call void @objc_msgSend(ptr %msg_result68, ptr %sel177, i1 false)
  %sel178 = call ptr @sel_registerName(ptr @sel_name.129)
  %msg_result179 = call ptr @objc_msgSend(ptr %msg_result68, ptr %sel178)
  %sel180 = call ptr @sel_registerName(ptr @sel_name.130)
  %msg_result181 = call ptr @objc_msgSend(ptr %msg_result179, ptr %sel180, double 4.800000e+01)
  %sel182 = call ptr @sel_registerName(ptr @sel_name.131)
  call void @objc_msgSend(ptr %msg_result181, ptr %sel182, i1 true)
  %sel183 = call ptr @sel_registerName(ptr @sel_name.132)
  %msg_result184 = call ptr @objc_msgSend(ptr %msg_result68, ptr %sel183)
  %sel185 = call ptr @sel_registerName(ptr @sel_name.133)
  %msg_result186 = call ptr @objc_msgSend(ptr %msg_result184, ptr %sel185, double 2.000000e+02)
  %sel187 = call ptr @sel_registerName(ptr @sel_name.134)
  call void @objc_msgSend(ptr %msg_result186, ptr %sel187, i1 true)
  %sel188 = call ptr @sel_registerName(ptr @sel_name.135)
  %msg_result189 = call ptr @objc_msgSend(ptr %msg_result68, ptr %sel188)
  %sel190 = call ptr @sel_registerName(ptr @sel_name.136)
  %msg_result191 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel190)
  %sel192 = call ptr @sel_registerName(ptr @sel_name.137)
  %msg_result193 = call ptr @objc_msgSend(ptr %msg_result189, ptr %sel192, ptr %msg_result191, double 0.000000e+00)
  %sel194 = call ptr @sel_registerName(ptr @sel_name.138)
  call void @objc_msgSend(ptr %msg_result193, ptr %sel194, i1 true)
  %sel195 = call ptr @sel_registerName(ptr @sel_name.139)
  %msg_result196 = call ptr @objc_msgSend(ptr %msg_result68, ptr %sel195)
  %sel197 = call ptr @sel_registerName(ptr @sel_name.140)
  %msg_result198 = call ptr @objc_msgSend(ptr %msg_result49, ptr %sel197)
  %sel199 = call ptr @sel_registerName(ptr @sel_name.141)
  %msg_result200 = call ptr @objc_msgSend(ptr %msg_result196, ptr %sel199, ptr %msg_result198, double 1.200000e+01)
  %sel201 = call ptr @sel_registerName(ptr @sel_name.142)
  call void @objc_msgSend(ptr %msg_result200, ptr %sel201, i1 true)
  %sel202 = call ptr @sel_registerName(ptr @sel_name.143)
  call void @objc_msgSend(ptr %msg_result99, ptr %sel202, i1 false)
  %sel203 = call ptr @sel_registerName(ptr @sel_name.144)
  %msg_result204 = call ptr @objc_msgSend(ptr %msg_result99, ptr %sel203)
  %sel205 = call ptr @sel_registerName(ptr @sel_name.145)
  %msg_result206 = call ptr @objc_msgSend(ptr %msg_result204, ptr %sel205, double 1.200000e+02)
  %sel207 = call ptr @sel_registerName(ptr @sel_name.146)
  call void @objc_msgSend(ptr %msg_result206, ptr %sel207, i1 true)
  %sel208 = call ptr @sel_registerName(ptr @sel_name.147)
  %msg_result209 = call ptr @objc_msgSend(ptr %msg_result99, ptr %sel208)
  %sel210 = call ptr @sel_registerName(ptr @sel_name.148)
  %msg_result211 = call ptr @objc_msgSend(ptr %msg_result209, ptr %sel210, double 2.000000e+02)
  %sel212 = call ptr @sel_registerName(ptr @sel_name.149)
  call void @objc_msgSend(ptr %msg_result211, ptr %sel212, i1 true)
  %sel213 = call ptr @sel_registerName(ptr @sel_name.150)
  %msg_result214 = call ptr @objc_msgSend(ptr %msg_result99, ptr %sel213)
  %sel215 = call ptr @sel_registerName(ptr @sel_name.151)
  %msg_result216 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel215)
  %sel217 = call ptr @sel_registerName(ptr @sel_name.152)
  %msg_result218 = call ptr @objc_msgSend(ptr %msg_result214, ptr %sel217, ptr %msg_result216, double 0.000000e+00)
  %sel219 = call ptr @sel_registerName(ptr @sel_name.153)
  call void @objc_msgSend(ptr %msg_result218, ptr %sel219, i1 true)
  %sel220 = call ptr @sel_registerName(ptr @sel_name.154)
  %msg_result221 = call ptr @objc_msgSend(ptr %msg_result99, ptr %sel220)
  %sel222 = call ptr @sel_registerName(ptr @sel_name.155)
  %msg_result223 = call ptr @objc_msgSend(ptr %msg_result68, ptr %sel222)
  %sel224 = call ptr @sel_registerName(ptr @sel_name.156)
  %msg_result225 = call ptr @objc_msgSend(ptr %msg_result221, ptr %sel224, ptr %msg_result223, double 1.200000e+01)
  %sel226 = call ptr @sel_registerName(ptr @sel_name.157)
  call void @objc_msgSend(ptr %msg_result225, ptr %sel226, i1 true)
  %sel227 = call ptr @sel_registerName(ptr @sel_name.158)
  call void @objc_msgSend(ptr %msg_result107, ptr %sel227, i1 false)
  %sel228 = call ptr @sel_registerName(ptr @sel_name.159)
  %msg_result229 = call ptr @objc_msgSend(ptr %msg_result107, ptr %sel228)
  %sel230 = call ptr @sel_registerName(ptr @sel_name.160)
  %msg_result231 = call ptr @objc_msgSend(ptr %msg_result229, ptr %sel230, double 4.800000e+01)
  %sel232 = call ptr @sel_registerName(ptr @sel_name.161)
  call void @objc_msgSend(ptr %msg_result231, ptr %sel232, i1 true)
  %sel233 = call ptr @sel_registerName(ptr @sel_name.162)
  %msg_result234 = call ptr @objc_msgSend(ptr %msg_result107, ptr %sel233)
  %sel235 = call ptr @sel_registerName(ptr @sel_name.163)
  %msg_result236 = call ptr @objc_msgSend(ptr %msg_result234, ptr %sel235, double 3.000000e+02)
  %sel237 = call ptr @sel_registerName(ptr @sel_name.164)
  call void @objc_msgSend(ptr %msg_result236, ptr %sel237, i1 true)
  %sel238 = call ptr @sel_registerName(ptr @sel_name.165)
  %msg_result239 = call ptr @objc_msgSend(ptr %msg_result107, ptr %sel238)
  %sel240 = call ptr @sel_registerName(ptr @sel_name.166)
  %msg_result241 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel240)
  %sel242 = call ptr @sel_registerName(ptr @sel_name.167)
  %msg_result243 = call ptr @objc_msgSend(ptr %msg_result239, ptr %sel242, ptr %msg_result241, double 0.000000e+00)
  %sel244 = call ptr @sel_registerName(ptr @sel_name.168)
  call void @objc_msgSend(ptr %msg_result243, ptr %sel244, i1 true)
  %sel245 = call ptr @sel_registerName(ptr @sel_name.169)
  %msg_result246 = call ptr @objc_msgSend(ptr %msg_result107, ptr %sel245)
  %sel247 = call ptr @sel_registerName(ptr @sel_name.170)
  %msg_result248 = call ptr @objc_msgSend(ptr %msg_result99, ptr %sel247)
  %sel249 = call ptr @sel_registerName(ptr @sel_name.171)
  %msg_result250 = call ptr @objc_msgSend(ptr %msg_result246, ptr %sel249, ptr %msg_result248, double 0.000000e+00)
  %sel251 = call ptr @sel_registerName(ptr @sel_name.172)
  call void @objc_msgSend(ptr %msg_result250, ptr %sel251, i1 true)
  %sel252 = call ptr @sel_registerName(ptr @sel_name.173)
  call void @objc_msgSend(ptr %msg_result120, ptr %sel252, i1 false)
  %sel253 = call ptr @sel_registerName(ptr @sel_name.174)
  %msg_result254 = call ptr @objc_msgSend(ptr %msg_result120, ptr %sel253)
  %sel255 = call ptr @sel_registerName(ptr @sel_name.175)
  %msg_result256 = call ptr @objc_msgSend(ptr %msg_result254, ptr %sel255, double 5.000000e+01)
  %sel257 = call ptr @sel_registerName(ptr @sel_name.176)
  call void @objc_msgSend(ptr %msg_result256, ptr %sel257, i1 true)
  %sel258 = call ptr @sel_registerName(ptr @sel_name.177)
  %msg_result259 = call ptr @objc_msgSend(ptr %msg_result120, ptr %sel258)
  %sel260 = call ptr @sel_registerName(ptr @sel_name.178)
  %msg_result261 = call ptr @objc_msgSend(ptr %msg_result259, ptr %sel260, double 1.836000e+02)
  %sel262 = call ptr @sel_registerName(ptr @sel_name.179)
  call void @objc_msgSend(ptr %msg_result261, ptr %sel262, i1 true)
  %sel263 = call ptr @sel_registerName(ptr @sel_name.180)
  %msg_result264 = call ptr @objc_msgSend(ptr %msg_result120, ptr %sel263)
  %sel265 = call ptr @sel_registerName(ptr @sel_name.181)
  %msg_result266 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel265)
  %sel267 = call ptr @sel_registerName(ptr @sel_name.182)
  %msg_result268 = call ptr @objc_msgSend(ptr %msg_result264, ptr %sel267, ptr %msg_result266, double 0.000000e+00)
  %sel269 = call ptr @sel_registerName(ptr @sel_name.183)
  call void @objc_msgSend(ptr %msg_result268, ptr %sel269, i1 true)
  %sel270 = call ptr @sel_registerName(ptr @sel_name.184)
  %msg_result271 = call ptr @objc_msgSend(ptr %msg_result120, ptr %sel270)
  %sel272 = call ptr @sel_registerName(ptr @sel_name.185)
  %msg_result273 = call ptr @objc_msgSend(ptr %msg_result107, ptr %sel272)
  %sel274 = call ptr @sel_registerName(ptr @sel_name.186)
  %msg_result275 = call ptr @objc_msgSend(ptr %msg_result271, ptr %sel274, ptr %msg_result273, double 0.000000e+00)
  %sel276 = call ptr @sel_registerName(ptr @sel_name.187)
  call void @objc_msgSend(ptr %msg_result275, ptr %sel276, i1 true)
  %sel277 = call ptr @sel_registerName(ptr @sel_name.188)
  call void @objc_msgSend(ptr %msg_result132, ptr %sel277, i1 false)
  %sel278 = call ptr @sel_registerName(ptr @sel_name.189)
  %msg_result279 = call ptr @objc_msgSend(ptr %msg_result132, ptr %sel278)
  %sel280 = call ptr @sel_registerName(ptr @sel_name.190)
  %msg_result281 = call ptr @objc_msgSend(ptr %msg_result279, ptr %sel280, double 5.000000e+01)
  %sel282 = call ptr @sel_registerName(ptr @sel_name.191)
  call void @objc_msgSend(ptr %msg_result281, ptr %sel282, i1 true)
  %sel283 = call ptr @sel_registerName(ptr @sel_name.192)
  %msg_result284 = call ptr @objc_msgSend(ptr %msg_result132, ptr %sel283)
  %sel285 = call ptr @sel_registerName(ptr @sel_name.193)
  %msg_result286 = call ptr @objc_msgSend(ptr %msg_result284, ptr %sel285, double 0x405E999999999999)
  %sel287 = call ptr @sel_registerName(ptr @sel_name.194)
  call void @objc_msgSend(ptr %msg_result286, ptr %sel287, i1 true)
  %sel288 = call ptr @sel_registerName(ptr @sel_name.195)
  %msg_result289 = call ptr @objc_msgSend(ptr %msg_result132, ptr %sel288)
  %sel290 = call ptr @sel_registerName(ptr @sel_name.196)
  %msg_result291 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel290)
  %sel292 = call ptr @sel_registerName(ptr @sel_name.197)
  %msg_result293 = call ptr @objc_msgSend(ptr %msg_result289, ptr %sel292, ptr %msg_result291, double 0.000000e+00)
  %sel294 = call ptr @sel_registerName(ptr @sel_name.198)
  call void @objc_msgSend(ptr %msg_result293, ptr %sel294, i1 true)
  %sel295 = call ptr @sel_registerName(ptr @sel_name.199)
  %msg_result296 = call ptr @objc_msgSend(ptr %msg_result132, ptr %sel295)
  %sel297 = call ptr @sel_registerName(ptr @sel_name.200)
  %msg_result298 = call ptr @objc_msgSend(ptr %msg_result120, ptr %sel297)
  %sel299 = call ptr @sel_registerName(ptr @sel_name.201)
  %msg_result300 = call ptr @objc_msgSend(ptr %msg_result296, ptr %sel299, ptr %msg_result298, double 0.000000e+00)
  %sel301 = call ptr @sel_registerName(ptr @sel_name.202)
  call void @objc_msgSend(ptr %msg_result300, ptr %sel301, i1 true)
  %sel302 = call ptr @sel_registerName(ptr @sel_name.203)
  call void @objc_msgSend(ptr %msg_result144, ptr %sel302, i1 false)
  %sel303 = call ptr @sel_registerName(ptr @sel_name.204)
  %msg_result304 = call ptr @objc_msgSend(ptr %msg_result144, ptr %sel303)
  %sel305 = call ptr @sel_registerName(ptr @sel_name.205)
  %msg_result306 = call ptr @objc_msgSend(ptr %msg_result304, ptr %sel305, double 5.000000e+01)
  %sel307 = call ptr @sel_registerName(ptr @sel_name.206)
  call void @objc_msgSend(ptr %msg_result306, ptr %sel307, i1 true)
  %sel308 = call ptr @sel_registerName(ptr @sel_name.207)
  %msg_result309 = call ptr @objc_msgSend(ptr %msg_result144, ptr %sel308)
  %sel310 = call ptr @sel_registerName(ptr @sel_name.208)
  %msg_result311 = call ptr @objc_msgSend(ptr %msg_result309, ptr %sel310, double 1.836000e+02)
  %sel312 = call ptr @sel_registerName(ptr @sel_name.209)
  call void @objc_msgSend(ptr %msg_result311, ptr %sel312, i1 true)
  %sel313 = call ptr @sel_registerName(ptr @sel_name.210)
  %msg_result314 = call ptr @objc_msgSend(ptr %msg_result144, ptr %sel313)
  %sel315 = call ptr @sel_registerName(ptr @sel_name.211)
  %msg_result316 = call ptr @objc_msgSend(ptr %msg_result40, ptr %sel315)
  %sel317 = call ptr @sel_registerName(ptr @sel_name.212)
  %msg_result318 = call ptr @objc_msgSend(ptr %msg_result314, ptr %sel317, ptr %msg_result316, double 0.000000e+00)
  %sel319 = call ptr @sel_registerName(ptr @sel_name.213)
  call void @objc_msgSend(ptr %msg_result318, ptr %sel319, i1 true)
  %sel320 = call ptr @sel_registerName(ptr @sel_name.214)
  %msg_result321 = call ptr @objc_msgSend(ptr %msg_result144, ptr %sel320)
  %sel322 = call ptr @sel_registerName(ptr @sel_name.215)
  %msg_result323 = call ptr @objc_msgSend(ptr %msg_result132, ptr %sel322)
  %sel324 = call ptr @sel_registerName(ptr @sel_name.216)
  %msg_result325 = call ptr @objc_msgSend(ptr %msg_result321, ptr %sel324, ptr %msg_result323, double 0.000000e+00)
  %sel326 = call ptr @sel_registerName(ptr @sel_name.217)
  call void @objc_msgSend(ptr %msg_result325, ptr %sel326, i1 true)
  %sel327 = call ptr @sel_registerName(ptr @sel_name.218)
  call void @objc_msgSend(ptr %msg_result31, ptr %sel327, ptr %msg_result40)
  %sel328 = call ptr @sel_registerName(ptr @sel_name.219)
  call void @objc_msgSend(ptr %msg_result31, ptr %sel328, { double, double } { double 3.400000e+02, double 4.800000e+02 })
  %sel329 = call ptr @sel_registerName(ptr @sel_name.220)
  call void @objc_msgSend(ptr %msg_result2, ptr %sel329, ptr %msg_result31)
  %sel330 = call ptr @sel_registerName(ptr @sel_name.221)
  call void @objc_msgSend(ptr %msg_result11, ptr %sel330, i1 false)
  %sel331 = call ptr @sel_registerName(ptr @sel_name.222)
  %msg_result332 = call ptr @objc_msgSend(ptr %msg_result11, ptr %sel331)
  %sel333 = call ptr @sel_registerName(ptr @sel_name.223)
  %msg_result334 = call ptr @objc_msgSend(ptr %msg_result332, ptr %sel333, double 2.000000e+02)
  %sel335 = call ptr @sel_registerName(ptr @sel_name.224)
  call void @objc_msgSend(ptr %msg_result334, ptr %sel335, i1 true)
  %sel336 = call ptr @sel_registerName(ptr @sel_name.225)
  %msg_result337 = call ptr @objc_msgSend(ptr %msg_result11, ptr %sel336)
  %sel338 = call ptr @sel_registerName(ptr @sel_name.226)
  %msg_result339 = call ptr @objc_msgSend(ptr %msg_result337, ptr %sel338, double 2.000000e+02)
  %sel340 = call ptr @sel_registerName(ptr @sel_name.227)
  call void @objc_msgSend(ptr %msg_result339, ptr %sel340, i1 true)
  %sel341 = call ptr @sel_registerName(ptr @sel_name.228)
  %msg_result342 = call ptr @objc_msgSend(ptr %msg_result11, ptr %sel341)
  %sel343 = call ptr @sel_registerName(ptr @sel_name.229)
  %msg_result344 = call ptr @objc_msgSend(ptr %msg_result2, ptr %sel343)
  %sel345 = call ptr @sel_registerName(ptr @sel_name.230)
  %msg_result346 = call ptr @objc_msgSend(ptr %msg_result342, ptr %sel345, ptr %msg_result344, double 0.000000e+00)
  %sel347 = call ptr @sel_registerName(ptr @sel_name.231)
  call void @objc_msgSend(ptr %msg_result346, ptr %sel347, i1 true)
  %sel348 = call ptr @sel_registerName(ptr @sel_name.232)
  %msg_result349 = call ptr @objc_msgSend(ptr %msg_result11, ptr %sel348)
  %sel350 = call ptr @sel_registerName(ptr @sel_name.233)
  %msg_result351 = call ptr @objc_msgSend(ptr %msg_result2, ptr %sel350)
  %sel352 = call ptr @sel_registerName(ptr @sel_name.234)
  %msg_result353 = call ptr @objc_msgSend(ptr %msg_result349, ptr %sel352, ptr %msg_result351, double -1.670000e+02)
  %sel354 = call ptr @sel_registerName(ptr @sel_name.235)
  call void @objc_msgSend(ptr %msg_result353, ptr %sel354, i1 true)
  %sel355 = call ptr @sel_registerName(ptr @sel_name.236)
  call void @objc_msgSend(ptr %msg_result31, ptr %sel355, i1 false)
  %sel356 = call ptr @sel_registerName(ptr @sel_name.237)
  %msg_result357 = call ptr @objc_msgSend(ptr %msg_result31, ptr %sel356)
  %sel358 = call ptr @sel_registerName(ptr @sel_name.238)
  %msg_result359 = call ptr @objc_msgSend(ptr %msg_result357, ptr %sel358, double 1.600000e+02)
  %sel360 = call ptr @sel_registerName(ptr @sel_name.239)
  call void @objc_msgSend(ptr %msg_result359, ptr %sel360, i1 true)
  %sel361 = call ptr @sel_registerName(ptr @sel_name.240)
  %msg_result362 = call ptr @objc_msgSend(ptr %msg_result31, ptr %sel361)
  %sel363 = call ptr @sel_registerName(ptr @sel_name.241)
  %msg_result364 = call ptr @objc_msgSend(ptr %msg_result362, ptr %sel363, double 3.400000e+02)
  %sel365 = call ptr @sel_registerName(ptr @sel_name.242)
  call void @objc_msgSend(ptr %msg_result364, ptr %sel365, i1 true)
  %sel366 = call ptr @sel_registerName(ptr @sel_name.243)
  %msg_result367 = call ptr @objc_msgSend(ptr %msg_result31, ptr %sel366)
  %sel368 = call ptr @sel_registerName(ptr @sel_name.244)
  %msg_result369 = call ptr @objc_msgSend(ptr %msg_result2, ptr %sel368)
  %sel370 = call ptr @sel_registerName(ptr @sel_name.245)
  %msg_result371 = call ptr @objc_msgSend(ptr %msg_result367, ptr %sel370, ptr %msg_result369, double 0.000000e+00)
  %sel372 = call ptr @sel_registerName(ptr @sel_name.246)
  call void @objc_msgSend(ptr %msg_result371, ptr %sel372, i1 true)
  %sel373 = call ptr @sel_registerName(ptr @sel_name.247)
  %msg_result374 = call ptr @objc_msgSend(ptr %msg_result31, ptr %sel373)
  %sel375 = call ptr @sel_registerName(ptr @sel_name.248)
  %msg_result376 = call ptr @objc_msgSend(ptr %msg_result2, ptr %sel375)
  %sel377 = call ptr @sel_registerName(ptr @sel_name.249)
  %msg_result378 = call ptr @objc_msgSend(ptr %msg_result374, ptr %sel377, ptr %msg_result376, double 1.870000e+02)
  %sel379 = call ptr @sel_registerName(ptr @sel_name.250)
  call void @objc_msgSend(ptr %msg_result378, ptr %sel379, i1 true)
  store ptr %msg_result2, ptr @g_root_view, align 8
  %cls380 = call ptr @objc_getClass(ptr @class_name.251)
  %sel381 = call ptr @sel_registerName(ptr @sel_name.252)
  %msg_result382 = call ptr @objc_msgSend(ptr %cls380, ptr %sel381)
  %sel383 = call ptr @sel_registerName(ptr @sel_name.253)
  %msg_result384 = call { { double, double }, { double, double } } @objc_msgSend(ptr %msg_result382, ptr %sel383)
  %cls385 = call ptr @objc_getClass(ptr @class_name.254)
  %sel386 = call ptr @sel_registerName(ptr @sel_name.255)
  %msg_result387 = call ptr @objc_msgSend(ptr %cls385, ptr %sel386)
  %sel388 = call ptr @sel_registerName(ptr @sel_name.256)
  %msg_result389 = call ptr @objc_msgSend(ptr %msg_result387, ptr %sel388, { { double, double }, { double, double } } %msg_result384)
  %cls390 = call ptr @objc_getClass(ptr @class_name.257)
  %sel391 = call ptr @sel_registerName(ptr @sel_name.258)
  %msg_result392 = call ptr @objc_msgSend(ptr %cls390, ptr %sel391)
  %sel393 = call ptr @sel_registerName(ptr @sel_name.259)
  call void @objc_msgSend(ptr %msg_result389, ptr %sel393, ptr %msg_result392)
  %cls394 = call ptr @objc_getClass(ptr @class_name.260)
  %sel395 = call ptr @sel_registerName(ptr @sel_name.261)
  %msg_result396 = call ptr @objc_msgSend(ptr %cls394, ptr %sel395)
  %sel397 = call ptr @sel_registerName(ptr @sel_name.262)
  %msg_result398 = call ptr @objc_msgSend(ptr %msg_result396, ptr %sel397)
  %loaded_view = load ptr, ptr @g_root_view, align 8
  %sel399 = call ptr @sel_registerName(ptr @sel_name.263)
  %msg_result400 = call ptr @objc_msgSend(ptr %msg_result398, ptr %sel399)
  %sel401 = call ptr @sel_registerName(ptr @sel_name.264)
  call void @objc_msgSend(ptr %msg_result400, ptr %sel401, ptr %msg_result392)
  %sel402 = call ptr @sel_registerName(ptr @sel_name.265)
  call void @objc_msgSend(ptr %msg_result400, ptr %sel402, ptr %loaded_view)
  %sel403 = call ptr @sel_registerName(ptr @sel_name.266)
  call void @objc_msgSend(ptr %loaded_view, ptr %sel403, i1 false)
  %sel404 = call ptr @sel_registerName(ptr @sel_name.267)
  %msg_result405 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel404)
  %sel406 = call ptr @sel_registerName(ptr @sel_name.268)
  %msg_result407 = call ptr @objc_msgSend(ptr %msg_result400, ptr %sel406)
  %sel408 = call ptr @sel_registerName(ptr @sel_name.269)
  %msg_result409 = call ptr @objc_msgSend(ptr %msg_result405, ptr %sel408, ptr %msg_result407)
  %sel410 = call ptr @sel_registerName(ptr @sel_name.270)
  call void @objc_msgSend(ptr %msg_result409, ptr %sel410, i1 true)
  %sel411 = call ptr @sel_registerName(ptr @sel_name.271)
  %msg_result412 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel411)
  %sel413 = call ptr @sel_registerName(ptr @sel_name.272)
  %msg_result414 = call ptr @objc_msgSend(ptr %msg_result400, ptr %sel413)
  %sel415 = call ptr @sel_registerName(ptr @sel_name.273)
  %msg_result416 = call ptr @objc_msgSend(ptr %msg_result412, ptr %sel415, ptr %msg_result414)
  %sel417 = call ptr @sel_registerName(ptr @sel_name.274)
  call void @objc_msgSend(ptr %msg_result416, ptr %sel417, i1 true)
  %sel418 = call ptr @sel_registerName(ptr @sel_name.275)
  %msg_result419 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel418)
  %sel420 = call ptr @sel_registerName(ptr @sel_name.276)
  %msg_result421 = call ptr @objc_msgSend(ptr %msg_result400, ptr %sel420)
  %sel422 = call ptr @sel_registerName(ptr @sel_name.277)
  %msg_result423 = call ptr @objc_msgSend(ptr %msg_result419, ptr %sel422, ptr %msg_result421)
  %sel424 = call ptr @sel_registerName(ptr @sel_name.278)
  call void @objc_msgSend(ptr %msg_result423, ptr %sel424, i1 true)
  %sel425 = call ptr @sel_registerName(ptr @sel_name.279)
  %msg_result426 = call ptr @objc_msgSend(ptr %loaded_view, ptr %sel425)
  %sel427 = call ptr @sel_registerName(ptr @sel_name.280)
  %msg_result428 = call ptr @objc_msgSend(ptr %msg_result400, ptr %sel427)
  %sel429 = call ptr @sel_registerName(ptr @sel_name.281)
  %msg_result430 = call ptr @objc_msgSend(ptr %msg_result426, ptr %sel429, ptr %msg_result428)
  %sel431 = call ptr @sel_registerName(ptr @sel_name.282)
  call void @objc_msgSend(ptr %msg_result430, ptr %sel431, i1 true)
  %sel432 = call ptr @sel_registerName(ptr @sel_name.283)
  call void @objc_msgSend(ptr %msg_result389, ptr %sel432, ptr %msg_result398)
  %sel433 = call ptr @sel_registerName(ptr @sel_name.284)
  call void @objc_msgSend(ptr %msg_result389, ptr %sel433)
  ret i8 1
}

define void @app_delegate_increment_state(ptr %0, ptr %1, ptr %2) {
entry:
  %state_current = load i64, ptr @g_state_value, align 4
  %state_next = add i64 %state_current, 1
  store i64 %state_next, ptr @g_state_value, align 4
  %formatted_state = call i32 (ptr, i64, ptr, ...) @snprintf(ptr @g_state_text_buffer, i64 32, ptr @state_format, i64 %state_next)
  %cls = call ptr @objc_getClass(ptr @class_name.286)
  %sel = call ptr @sel_registerName(ptr @sel_name.287)
  %msg_result = call ptr @objc_msgSend(ptr %cls, ptr %sel)
  %sel1 = call ptr @sel_registerName(ptr @sel_name.288)
  %msg_result2 = call ptr @objc_msgSend(ptr %msg_result, ptr %sel1, ptr @g_state_text_buffer)
  %state_label = load ptr, ptr @g_state_label, align 8
  %sel3 = call ptr @sel_registerName(ptr @sel_name.289)
  call void @objc_msgSend(ptr %state_label, ptr %sel3, ptr %msg_result2)
  ret void
}

declare i32 @snprintf(ptr, i64, ptr, ...)

declare i32 @UIApplicationMain(i32, ptr, ptr, ptr)
