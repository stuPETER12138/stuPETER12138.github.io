# VIM 基础教程

[IdeaVim](https://github.com/JetBrains/ideavim)  Tutor: As you go through this tutor, do not try to memorize, learn by usage.

### 壹 

~~使用鼠标来移动光标~~

当处于 **NORMAL** 模式时，使用 `h`，`j`，`k`，`l` 来移动光标，如下表；

| h  | j  | k  | l  |
|:--:|:--:|:--:|:--:|
| 向左 | 向下 | 向上 | 向右 |

多加练习，开始用他们辅助文本编辑吧！

1. 如何删除：

   按一下 `x` 将删除光标下的一个字符。

2. 插入模式（**INSERTION**）:

   按一下 `i` 以进入插入模式。

   按一下 `ESC` 以返回 **NORMAL** 模式。

3. 行末添加文本：

   按一下 `Caps` + `a` 将光标移至行末并切换为插入模式。（换用 `i` 也有不一样的效果呢）

### 贰

1. 依然处于 **NORMAL** 模式下，当光标位于一个单词的首字母时，按一下 `d` + `w` 可以删除一个单词。而中文文本下，光标位于句首时会删掉一整句话。

   值得注意的是，当按下 `d` 后，它将出现在状态栏，*等待下一步指令*（如 `w`）。如果发现第二个字母打错了，别慌，按一下 `ESC` 就可以重新开始。

2. 额外地，按一下 `d` + `$` 则可以将一行文本中光标以后的部分全部删掉。
对于一个单词，按一下 `d` + `e` 则可以删除该单词中光标以后的部分（包括光标内容）。

3. 用数字可以表示步长。例如，按一下 `3` + `w` 可以将光标移动到向右第三个单词的首字母；
按一下 `2` + `e` 可以将光标移动到向右第二个单词的尾字母。
按一下 `0` 使光标返回到该行首字母。

4. 遇到单词是大写字母时，则可以将数字与 d 指令搭配使用。例如，按一下 `d` + `3` + `w` 可以删除连续三个单词（不论大小写）。

5. 按两下 `d` 可以删除一整行，按一下 `2` 加两下 `d` 可以删除两行。

6. 按一下 `u` 可以撤回，按一次 `Ctrl` + `r` 可以重做。

### 叁

1. 按一下 `p` 可以把前一步删掉的文本添加回光标后。

2. 按一下 `r` + ` ` 可以将光标下的字母替换掉。

3. `c` 的用法：
    
   按一下 `c` + `e` 以删掉单词在光标后的部分，**并进入插入模式**。
   
   与 `d` 的用法基本一致。

### 肆

1. 按一下 `Caps` + `g` 以将光标移动到文件最后一行句首。

   按两下 `g` 以移动光标至文件第一行句首。

   输入行数再按一下 `Caps` + `g` 或按两下 `g` 以将光标移动到指定行。

2. 按一下 `/ (?)` 以进入正向（反向）搜索。
   再按一下 `n (N)` 以正向（反向）查找下一个匹配对象。

3. 匹配括号。在一个括号处按一下 `%` 可以将光标移动到与该括号匹配的另一括号处。

   > 对于 debug 十分有效

4. 敲入 `:s/old/new` 可以将该行第一个目标单词替换为新单词。

   敲入 `:s/old/new/g` 则代表替换该行中所有的目标单词。

   敲入 `:%s/old/new/g` 则代表替换该文件中所有的目标单词。

   敲入 `:%s/old/new/gc` 则代表查找该文件中所有的目标单词，并提示是否替换。

   敲入 `:n1,n2s/old/new/g` 则代表替换第 n1 行至第 n2 行中所有的目标单词。 

### 伍

1. 按一下 `o` 以向下新添加一行，并进入插入模式。

   按一下 `Caps` + `o` 以向上新添加一行，并进入插入模式。

2. 按一下 `a` 以在光标后插入文本。

   按一下 `Caps` + `a` 以在行尾插入文本。

3. 按一下 `e` 可以将光标移动到单词末尾。

4. 按一下 `Caps` + `r` 进入替换模式，将光标位置的字符替换成新输入的字符。

5. 按一下 `v` 进入**视觉模式**，移动光标选择文本，按一下 `y` 以复制选中文本。

   按一下 `y` + `w` 以复制一个单词。

   按两下 `y` 复制一整行文本。

   按一下 `p` 以粘贴文本。

6. 敲入 `:set (no)xxx`，为搜索或替换添加附加条件：

   可选择的 `xxx` 有：

   - `ic` 忽略大小写
   - `hls` 高亮所有匹配项
   - `is` 额外显示搜索内容的部分匹配项

   在搜索或替换命令后敲入 `\c` 使上述条件只生效一次。

### 戛然而止

至此，ideavim 教程结束。

不过 ideavim 的学习却远不止于此。官方教程中还涉及到了 ideavimrc 启动脚本的创建，vim 基础教学，以及一些推荐书籍。
感兴趣的同学请自行到 IdeaVim 官方学习。
