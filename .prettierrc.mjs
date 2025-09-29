export default {
    $schema: "https://json.schemastore.org/prettierrc",
    /**
     * 行尾分号
     */
    semi: true,

    /**
     * 使用单引号代替双引号
     */
    singleQuote: false,

    /**
     * 缩进空格数
     */
    tabWidth: 4,

    /**
     * 多行时尽可能打印尾随逗号
     * none - 无尾随逗号
     * es5  - 在ES5中有效的尾随逗号（对象、数组等）
     * all  - 尽可能使用尾随逗号
     */
    trailingComma: "es5",

    /**
     * 每行代码最大长度
     */
    printWidth: 200,

    /**
     * 箭头函数参数括号
     * avoid - 能省略括号时就省略
     * always - 总是有括号
     */
    arrowParens: "avoid",

    /**
     * 对象大括号内的首尾空格
     */
    bracketSpacing: true,

    /**
     * 行结束符
     * lf    - 仅换行（\n），常见于 Linux 和 macOS
     * crlf  - 回车符 + 换行符（\r\n），常见于 Windows
     * cr    - 仅回车符（\r），很少使用
     * auto  - 维护现有的行尾
     */
    endOfLine: "lf",

    /**
     * HTML 空白敏感度
     * css   - 遵守 CSS display 属性的默认值
     * strict- 空白敏感
     * ignore- 空白不敏感
     */
    htmlWhitespaceSensitivity: "ignore",

    singleAttributePerLine: false,

    /**
     * Vue 文件中 script 和 style 的缩进
     */
    vueIndentScriptAndStyle: false,

    bracketSameLine: true,
};
