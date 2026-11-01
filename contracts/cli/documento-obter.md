---
type: CommandContract
title: Ler documento SEI
operation: documento.obter
command: ies documento DOC
effect: read
implementation: stub
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Ler documento SEI

`ies documento DOC [DOC ...]` lê um ou mais documentos pelo número SEI
visível. A implementação atual permanece um stub sem acesso à rede.

## Entradas

- `DOC [DOC ...]`: um ou mais números de documento.
- `--processo NUP`: contexto opcional, quando informado.
- `--versao N`: índice positivo de versão histórica solicitado; continua indisponível sem árvore e sessão Web verificadas. Não implica listagem pública.
- `--sem-copia`: leitura sem preparação de cópia local; hoje não altera o stub.
- `--saida DIR`: diretório de saída para conteúdo baixado.
- `--ocultar-inacessiveis`: não expor documentos que a sessão não pode acessar.
- `--com-propriedades`: inclui os metadados do cadastro do documento.
- `--json` e `--unidade SIGLA`: opções globais.

## Saída prevista

`LoteConteudosDocumento` representa os conteúdos e metadados obtidos.
No Todos, múltiplos documentos de um mesmo processo podem ser agrupados
num ZIP quando a própria interface do SEI oferece essa operação.
Não presumir essa oferta em instalações que não a apresentam.

## Falhas e evidências

O shim de entrada interpreta `DOC` como número SEI composto apenas por\ndígitos ASCII, preserva zeros à esquerda e não consulta a instalação.\nEntrada inválida retorna `entrada_invalida` com subcódigo tipado e saída 2.

Espaços externos comuns podem ser aparados. Caracteres de controle
(CR, LF, tab ou outros), **inclusive nas extremidades da entrada
original**, são recusados com variante Rust `Controle` e subcódigo
`numero_documento_controle`, antes de qualquer normalização.
Caracteres não ASCII/dígitos misturados continuam retornando
`numero_documento_nao_numerico`; nunca converter números Unicode.\nUma entrada válida ainda retorna `nao_implementado` e
código 1. Arquivo inacessível, ausência de documento e sessão inválida
precisarão de contratos de erro específicos antes de implementar HTTP.

[documento.obter](../evidencias/documento-obter.md) e
[formulários e payloads](../../docs/como-o-sei-funciona/formularios.md)
fornecem evidências de partida. Não inferir autorização ou saída sem prova.

## Recorte adicional: histórico de versões reproduzido offline

A ferramenta auxiliar `ies replay CENARIO --documento --historico-versoes`
aceita número SEI solicitado, ID interno explicitamente observado,
resposta GET `documento_versao_listar` de cassette conferida e,
opcionalmente, `--versao-documento N`. Apresenta somente maior versão
efetivamente observada e metadados mínimos da versão selecionada;
**não implementa** `ies documento DOC --versao N` nem cria
`listar-versoes`. A relação entre número SEI e ID interno permanece
não comprovada e é exposta como `numero_documento_confirmado: false`.
Links assinados não são publicados nem executados. Refs #88.


## Identidade textual de HTML reproduzido offline (Refs #60)

Nas respostas HTML com `head`, apenas o `title` dentro desse
cabeçalho define a identidade `SEI/instalação - DOC - tipo`. O
`title` de um SVG dentro do `body` descreve o gráfico e **não**
deve substituir nem duplicar o título do documento. Mais de um
cabeçalho HTML, ou mais de um título dentro do mesmo cabeçalho,
recusa a interpretação com erro Rust tipado. Um cabeçalho sem título
não é corrigido silenciosamente por um título encontrado no corpo.

Para manter compatibilidade com fragmentos de cassette sem `head`,
o parser offline conserva a aceitação explícita de um título único
nesse caso, sem considerar isso prova de um documento Web completo.
O recorte não realiza leitura remota, não modifica o stub
`ies documento DOC` e não substitui a verificação de número/ID,
sessão, origem, MIME e versão antes de uma futura consulta live.

## Legenda e cobertura do histórico Web (Refs #88 e #60)

A tabela de versões só pode ter **uma legenda `caption`**. Mesmo que
ambas declarem o mesmo total, duas legendas são ambíguas e o parser
falha com a variante Rust `LegendasDoHistoricoRepetidas` e subcódigo
`legendas_historico_versoes_repetidas`. Não escolher a primeira legenda
para certificar cobertura, sobretudo se uma legenda externa preceder
`Lista de Versões`. A contagem continua confrontada às linhas
realmente observadas; isso é replay offline e não consulta live.
