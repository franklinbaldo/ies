---
type: CommandContract
title: Ler documento SEI
operation: documento.obter
command: ies documento DOC
effect: read
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Ler documento SEI

`ies documento DOC [DOC ...]` lê um ou mais documentos pelo número SEI
visível. A leitura Web de documentos internos (texto e metadados, pela
pesquisa rápida, a árvore e o link assinado do conteúdo) está no
[contrato da leitura Web](documento-obter-web.md) (Refs #60, fatia 1);
`--com-propriedades` lê o cadastro publicado na barra do documento e
`--ocultar-inacessiveis` omite somente conteúdo não publicado, conforme
o [contrato Web](documento-obter-web.md).
Versões, `--saida` e HTML privado estão descritos abaixo.

## Entradas

- `DOC [DOC ...]`: um ou mais números de documento.
- `--processo NUP`: contexto opcional, quando informado.
- `--versao N`: lê a versão N pela lista que a barra do documento publica ([contrato de versões](documento-obter-versoes.md), Refs #88). Não implica listagem pública.
- `--sem-cache`: desativa o cache gerenciado de cada documento lido; `IES_CACHE_DOCUMENTOS=0` tem o mesmo efeito. A saída opcional segue independente.
- `--saida DIR`: grava a foto `DOC.vN.html` de cada interno (o HTML
  servido da versão lida, idempotente) e o arquivo original do externo
  ([versões e foto](documento-obter-versoes.md), #207). `--sem-copia`
  saiu em #207.
- `--ocultar-inacessiveis`: não expor documentos que a sessão não pode acessar.
- `--com-propriedades`: inclui os metadados do cadastro do documento.
- `--json` e `--unidade SIGLA`: opções globais.

## Saída prevista

`LoteConteudosDocumento` representa os conteúdos e metadados obtidos.
No Todos, múltiplos documentos de um mesmo processo podem ser agrupados
num ZIP quando a própria interface do SEI oferece essa operação.
Não presumir essa oferta em instalações que não a apresentam.

## Falhas e evidências

O shim de entrada interpreta `DOC` como número SEI composto apenas por\ndígitos ASCII, preserva zeros à esquerda e não consulta a instalação.\nEntrada inválida é erro do clap, saída 2, antes de qualquer arquivo ou rede.

Espaços externos comuns podem ser aparados. Caracteres de controle
(CR, LF, tab ou outros), **inclusive nas extremidades da entrada
original**, são recusados com o erro `Controle` e subcódigo
`numero_documento_controle`, antes de qualquer normalização.
Caracteres não ASCII/dígitos misturados continuam retornando
`numero_documento_nao_numerico`; nunca converter números Unicode.\nUma entrada válida segue para a [leitura Web](documento-obter-web.md),
que tipa sessão, ausência, inacessibilidade e cancelamento, lê o
documento externo e aceita `--processo`, `--versao`, `--saida` e
`--ocultar-inacessiveis` (somente ausência observada de `.src` é
omitida no lote, com requisições contabilizadas). `--com-propriedades` acrescenta um GET somente quando a barra publica
link de cadastro e registra a ação consultada em `fonte`; sem link,
retorna propriedades nulas com motivo, sem GET extra.

[documento.obter](../evidencias/documento-obter.md) e
[formulários e payloads](../../docs/como-o-sei-funciona/formularios.md)
fornecem evidências de partida. Não inferir autorização ou saída sem prova.

## Recorte retirado: histórico de versões reproduzido offline

`ies replay CENARIO --documento --historico-versoes` saiu com o subcomando
`replay` (05/11/2026; [decisão](../../docs/decisoes/acervo/replay-fora-da-cli.md)).
O leitor das listas `documento_versao_listar` segue coberto pelo replay do
acervo `processo_obter::acervo_versoes` (`scripts/replay acervo_versoes`) e
pelos testes sintéticos de `documento_versoes`. A leitura Web
`ies documento DOC --versao N` está implementada (Refs #88); não há
subcomando `listar-versoes`. A retirada de `replay` (#208) não retirou
a opção Web de versão.

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
Os testes offline desse leitor não realizam leitura remota nem
substituem as verificações de número/ID, sessão, origem, MIME e versão
do fluxo Web implementado em `ies documento DOC`.

## Legenda e cobertura do histórico Web (Refs #88 e #60)

A tabela de versões só pode ter **uma legenda `caption`**. Mesmo que
ambas declarem o mesmo total, duas legendas são ambíguas e o parser
falha com o erro `LegendasDoHistoricoRepetidas` e subcódigo
`legendas_historico_versoes_repetidas`. Não escolher a primeira legenda
para certificar cobertura, sobretudo se uma legenda externa preceder
`Lista de Versões`. A contagem continua confrontada às linhas
realmente observadas; isso é replay offline e não consulta live.

## Leitura opt-in de HTML privado já capturado (Refs #60)

`ies documento DOC --html-privado ARQUIVO [--json]` interpreta **sem rede**
um único documento HTML obtido previamente pelo operador autorizado.
Exige arquivo regular privado (0600 no Unix), não segue symlink final
no Linux, limita a 8 MB e aplica o mesmo leitor seguro de arquivos
da sessão. Exige número SEI exato, inclusive zeros à esquerda; título
único SEI e texto visível do corpo, sem `script`, `style` ou `iframe`.
Erros de fonte, título, conteúdo e número diferente são erros tipados
com subcódigos próprios; falhas não expõem HTML nem tokens.

A saída é `escopo: html_privado_offline`, `autenticacao_remota: false`
e `versao_atual: null`. Não afirmar atualidade, assinatura, integridade
criptográfica, identidade da sessão nem versão do documento. Opções
remotas como `--versao` ou `--com-propriedades` são recusadas
no modo local, assim como vários DOCs (um arquivo só). Sem
`--html-privado`, `ies documento DOC` faz a [leitura Web](documento-obter-web.md).
Este modo local não faz login, GET, POST nem escrita no SEI.


## Exportação de texto do HTML privado (Refs #60)

`ies documento DOC --html-privado ARQUIVO --saida DIR` extrai o texto
somente depois de validar a identidade exata e o conteúdo offline.
`DIR` deve existir e recebe `DOC.txt` criado com permissão privada
0600 no Unix, sem sobrescrever arquivos nem seguir symlinks no destino.
Não cria diretórios, não altera o HTML capturado e nunca faz requisições
ao SEI. Um erro de gravação é tipado e não publica conteúdo.

Na exportação, a saída JSON informa documento, escopo
`html_privado_texto_exportado_offline`, bytes e
`versao_atual: null`, mas **não** devolve o texto no terminal.
Sem `--saida`, permanece a leitura offline de texto existente.
A `--saida DIR` da [leitura Web](documento-obter-web.md) não grava
`DOC.txt`: grava a foto `DOC.vN.html` (#207).

## Texto visível versus decoração HTML (Refs #60)

Na extração de texto, a identidade do documento continua sendo lida
exclusivamente do título HTML válido. No `body`, os conteúdos das
subárvores `svg` (ícones e descrições gráficas) e `template`
(fragmentos não renderizados) **não são texto do documento**.
Seu `title` ou texto interno não pode contaminar a cópia `.txt`.
O conteúdo de parágrafos visíveis mantém a ordem. Se restar apenas
decoração, a extração recusa com `ErroCorpoDocumento::CorpoVazio`,
sem emitir arquivo de saída.

A regra se aplica tanto à leitura `--html-privado` quanto à
exportação `--saida`, compartilhando o mesmo parser.
É verificação estrutural offline de HTML privado, sem supor que
a leitura Web autenticada esteja implementada ou validada.

## Destino privado (Refs #60, revisto em 05/11/2026)

O caminho de saída de `--html-privado ... --saida DIR` é
explicitamente escolhido pelo operador, e o modo dele é
responsabilidade do operador: o IES não o confere. Link simbólico no
caminho, inclusive o próprio `DIR` ou um ancestral (`/home` →
`/var/home` no Silverblue, `/var` → `/private/var` no macOS), é aceito:
o diretório é resolvido e o `DOC.txt` vai para o diretório real. O nome
final nunca é seguido: um link plantado em `DIR/DOC.txt` recusa a
gravação (`arquivo_texto_documento_ja_existe`) sem escrever através
dele. Diretório ausente continua gerando erro próprio; saída continua
com 0600 e sem sobrescrita
([decisão](../../docs/decisoes/arquivos-privados/gravacao-privada.md#modelo-de-ameaça)).
Nenhum acesso Web é acrescentado.

## Fragmentos inertes com código não executado (Refs #60)

Na leitura `--html-privado`, somente conteúdo executável presente na
árvore **visível** do `body` (`script`, `style`, `iframe` ou `noscript`)
recusa a extração com o erro tipado
`ErroCorpoDocumento::CorpoComConteudoExecutavel`.
Um `script` ou `iframe` alojado exclusivamente em `template`/`svg`
ignorados não invalida um parágrafo visível legítimo nem vaza o
conteúdo do fragmento: a subárvore toda é descartada antes da
verificação de executáveis. A ausência de texto visível continua
`ErroCorpoDocumento::CorpoVazio`. A regra é local/offline e não altera
o número de requisições ao SEI (zero), identidade, MIME ou autorização.

## Cabeçalho sem `head` não aceita títulos dentro do conteúdo (Refs #60)

Alguns fragmentos HTML históricos têm um `title` no nível externo,
sem `head`; preservar essa possibilidade de replay local. Porém um
`title` presente **somente** em `body`, `svg` ou `template` não
identifica o documento SEI, ainda que seu texto pareça um título
`SEI/instalação - DOC - Tipo`. Nessa situação a função de identidade
falha com o erro tipado existente `ErroTituloDocumento::TituloAusente`,
sem exportar conteúdo ou inferir documento pelo corpo. Dois títulos
externos válidos seguem rejeitados por `TitulosDeDocumentoRepetidos`.
A regra não abre URLs, não consulta a instalação, não modifica a
sintaxe da CLI e não prova autenticidade do HTML previamente capturado.
