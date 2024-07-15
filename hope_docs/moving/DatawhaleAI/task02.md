---
date: 2024-07-15
title: Datawhale AI 夏令营 Task 2 学习笔记
category: 
    - 我做
tag: 
    - datawhale
    - AI
---

从 baseline 代码[^1]详解入门深度学习

### 写在前面
通常我们基于神经网络解决机器翻译任务的流程如下：

![流程图](https://cdn.jsdelivr.net/gh/stuPETER12138/picgopic@latest/pictrues/20240715195725.png)

### 关于环境配置
`魔塔`平台已经提供了 Ubutu22.04 + CUDA12.1.0 + python3.10 + pytorch2.3.0 + TensorFlow2.14.0 的预装环境。不过为实现 Task 2，我们还需要额外添加一些库。

安装 `torchtext`, `jieba`, `sacrebleu`: 

```python
pip install torchtext    
pip install jieba
pip install sacrebleu
```

安装`spacy`，这会是比较“麻烦”的一步。

- 网络顺畅请使用：https://spacy.io/usage 。（毕竟是外国网站，速度可想而知
- 更推荐使用：离线安装。首先在终端查看更适合自己体质的 spacy 版本：`pip show spacy`。然后去找[对应版本的语言包](https://github.com/explosion/spacy-models/releases)。接下来，将下载到本地的压缩包上传到魔搭平台上的 dataset 目录下（文件较大，上传需要一定时间）。最后，一键安装：`pip install ../dataset/en_core_web_[一个代号]-[版本号]-py3-none-any.whl`

至此，我们便可以在平台上跑代码了。
 
### 关于数据处理
数据处理可以保证模型有效的学习到源语言到目标语言的映射。常见的步骤有：清洗与规范化，分词，构建词汇表与词向量，序列截断与填充，添加特殊标记，数据增强，数据分割等。

值得注意的是，在 baseline 中，使用`jieba`与`spacy`分别为中英文进行分词。

```python
en_tokenizer = get_tokenizer('spacy', language='en_core_web_trf')
zh_tokenizer = lambda x: list(jieba.cut(x))
```

并构建了词汇表，即从训练数据中收集所有出现过的词汇，构建词汇表，并为每个词分配一个唯一的索引。

同时添加了特殊标记：
1. 在序列两端添加`<BOS>`（Sequence Start）和`<EOS>`（Sequence End）标记，帮助模型识别序列的起始和结束。
2. 为不在词汇表中的词添加`<UNK>`（Unknown）标记，使模型能够处理未见过的词汇。

```python
def build_vocab(data: List[Tuple[List[str], List[str]]]):
    en_vocab = build_vocab_from_iterator(
        (en for en, _ in data),
        specials=['<unk>', '<pad>', '<bos>', '<eos>']
    )
    zh_vocab = build_vocab_from_iterator(
        (zh for _, zh in data),
        specials=['<unk>', '<pad>', '<bos>', '<eos>']
    )
    en_vocab.set_default_index(en_vocab['<unk>'])
    zh_vocab.set_default_index(zh_vocab['<unk>'])
    return en_vocab, zh_vocab

class TranslationDataset(Dataset):
    def __init__(self, data: List[Tuple[List[str], List[str]]], en_vocab, zh_vocab):
        self.data = data
        self.en_vocab = en_vocab
        self.zh_vocab = zh_vocab

    def __len__(self):
        return len(self.data)

    def __getitem__(self, idx):
        en, zh = self.data[idx]
        en_indices = [self.en_vocab['<bos>']] + [self.en_vocab[token] for token in en] + [self.en_vocab['<eos>']]
        zh_indices = [self.zh_vocab['<bos>']] + [self.zh_vocab[token] for token in zh] + [self.zh_vocab['<eos>']]
        return en_indices, zh_indices
```

在处理批次数据时，进行序列填充，确保所有序列长度相同。通常使用`<PAD>`标记填充。

```python
def collate_fn(batch):
    en_batch, zh_batch = [], []
    for en_item, zh_item in batch:
        if en_item and zh_item:  # 确保两个序列都不为空
            # print("都不为空")
            en_batch.append(torch.tensor(en_item))
            zh_batch.append(torch.tensor(zh_item))
        else:
            print("存在为空")
    if not en_batch or not zh_batch:  # 如果整个批次为空，返回空张量
        return torch.tensor([]), torch.tensor([])
    en_batch = nn.utils.rnn.pad_sequence(en_batch, batch_first=True, padding_value=en_vocab['<pad>'])
    zh_batch = nn.utils.rnn.pad_sequence(zh_batch, batch_first=True, padding_value=zh_vocab['<pad>'])
    return en_batch, zh_batch
```

### 关于模型训练
**编码器-解码器框架**在神经机器翻译中有着重要地位。它可以将源语言编码为类似信息传输中的数字信号，然后利用解码器对其进行转换，生成目标语言。

![一个运用编码器解码器结构的例子](https://cdn.jsdelivr.net/gh/stuPETER12138/picgopic@latest/pictrues/20240715200956.png)

- 编码器？在当今主流的神经机器翻译系统中，编码器由词嵌入层和中间网络层组成。当输入一串单词序列时，词嵌入层（embedding）会将每个单词映射到多维实数表示空间，这个过程也被称为词嵌入。之后中间层会对词嵌入向量进行更深层的抽象，得到输入单词序列的中间表示。
- 解码器？解码器的结构基本上和编码器是一致的，在基于循环神经网络的翻译模型中，解码器只比编码器多了输出层，用于输出每个目标语言位置的单词生成概率。而在基于自注意力机制的翻译模型中，除了输出层，解码器还比编码器多一个编码­解码注意力子层，用于帮助模型更好地利用源语言信息。

注意力机制！注意力机制的引入使得不再需要把原始文本中的所有必要信息压缩到一个向量当中。

![引入注意力机制的循环神经网络机器翻译架构](https://cdn.jsdelivr.net/gh/stuPETER12138/picgopic@latest/pictrues/20240715213100.png)

传统的 Seq2Seq 模型在解码阶段仅依赖于编码器产生的最后一个隐藏状态，这在处理长序列时效果不佳。注意力机制允许解码器在生成每个输出词时，关注编码器产生的所有中间状态，从而更好地利用源序列的信息。具体来说，给定源语言序列经过编码器输出的向量序列 $h_{1},h_{2},h_{3},...,h_{m}$，注意力机制旨在依据解码端翻译的需要，自适应地从这个向量序列中查找对应的信息。

值得注意的是，baseline 的代码中实现了一个经典的序列到序列（Seq2Seq）模型，中间层使用的 GRU 网络，并且网络中加入了注意力机制（Attention Mechanism）。

### 那么，何为 GRU?


### 参考资料
[Task2：从baseline代码详解入门深度学习](https://datawhaler.feishu.cn/wiki/PztLwkofsi95oak2Iercw9hkn2g?from=from_copylink)

[^1]: [Task2-baseline.ipynb](https://datawhaler.feishu.cn/wiki/PztLwkofsi95oak2Iercw9hkn2g#WtUhdX63OofXNdxp77NclTVGntb)
