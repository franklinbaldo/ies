---
type: Contract
title: Ler documento pela Web
description: ies documento DOC lê pela sessão Web o texto e os metadados de documentos internos pelo caminho mais barato (processo informado, índice local ou pesquisa rápida do número SEI), a árvore e o link assinado do conteúdo, com recusa tipada para os demais casos
---

# `ies documento DOC` pela Web (Refs #60, fatias 1 e 2)

Complementa o [contrato de `documento.obter`](documento-obter.md).
`ies documento DOC [DOC ...] [--processo NUP] [--saida DIR] [--json]
[--unidade SIGLA]` usa a sessão de `ies login` e lê, na ordem pedida, o
conteúdo de cada documento **interno** (editor do SEI) pelo número SEI.
`--versao N`, `versao_atual` e a foto `DOC.vN.html` de `--saida`
estão no [contrato de versões](documento-obter-versoes.md)
(Refs #88, #207), sobre qualquer caminho que abriu a árvore.
Com `--com-propriedades`, a CLI segue o link de cadastro **publicado na
barra do documento**: `documento_alterar` ou `documento_consultar`
(nativo em processo fechado), `documento_alterar_recebido` ou
`documento_consultar_recebido` (externo). A forma de consulta em
processo fechado e os rádios de nível desabilitados foram observados
na UFRJ 4.1 (cassette `documento-consultar-fechado-ufrj`).
Confere ação, IDs de documento e processo, lê `frmDocumentoCadastro`
com um único GET adicional (sem POST) e publica `propriedades` com
`fonte` igual à ação observada, `tipo`, `descricao`,
`nome_na_arvore`, `observacoes` e `nivel_de_acesso`.
Sem qualquer link de cadastro com `--com-propriedades`: falha tipada
`PropriedadesNaoPublicadasNaBarra` (saída 1, nenhum GET extra, nada
publicado como `null`). Sem a flag, a chave `propriedades` não sai.
Com `--anexo` e sem o cadastro do anexo: `ConteudoNaoPublicadoNaArvore`.
Formulário ilegível ou identidade divergente são recusas tipadas.
Não fabricar link de consulta nem tratar ausência de link como ausência
do documento.
`--ocultar-inacessiveis` é atendido na leitura Web: em lote, omite apenas
um documento cujo nó foi comprovado mas não publicou `.src`, sem suprimir
cancelamento, falha da sessão ou erro de identidade. `--html-privado`
segue o recorte local.
O documento **externo** (`.src` `documento_download_anexo`: PDF, TXT,
HTML recebido) é lido pelo [GET binário](documento-obter-externo.md)
(fatia 3); o subcódigo `documento_externo_por_download_de_anexo` foi
aposentado na fatia 3 de #60 (leitura do anexo externo).

## Caminho e requisições

Por documento, sem retry e sem seguir redirect, o primeiro que se aplica:

1. **`--processo NUP`:** o processo pelo caminho mais barato de
   [`processo NUP`](processo-obter-arvore.md) (link da árvore guardado:
   1 GET); na árvore, o **único** `DOCUMENTO` com o número SEI (a15)
   pedido; GET do `.src` dele. Custo: o de `processo NUP` + 1.
2. **Índice** ([ADR 0003](../../docs/arquitetura/0003-indice-local.md)):
   toda árvore lida (`processo NUP` e `documento DOC`) ensina número SEI
   ↔ (`id_procedimento` P, `id_documento` M). Com o palpite e o NUP de P
   no índice: (a) link da árvore de P guardado nesta geração e unidade
   (2 requisições: árvore e conteúdo); (b) link `procedimento_trabalhar`
   de P guardado (3: tela, árvore, conteúdo); (c) a entrada sem
   assinatura `procedimento_trabalhar&id_procedimento=P&id_documento=M`
   da lista branca, só onde nenhuma das duas formas foi recusada: 302
   para a mesma ação assinada, sem `acao_origem`, com os **dois** ids
   iguais aos pedidos, `infra_hash` de 64 hex, unidade e sistema; tela que
   seleciona M no processo P; árvore; conteúdo (4). Na árvore, o
   documento é o único nó com o número pedido (servidor vence: id
   diferente é reaprendido). Palpite desmentido (número fora da árvore,
   302 ou tela de outro id, árvore de outro NUP) é esquecido e o comando
   cai **uma vez** no passo 3.
3. **Pesquisa rápida** do número SEI exatamente como pedido: POST ao
   `action` guardado no índice ou, sem ele, GET do Controle e POST pelo
   `form#frmProtocoloPesquisaRapida` servido (o Controle lido é colhido);
   302 para `procedimento_trabalhar` com `id_protocolo` (o id do
   documento), conferido como no `processo NUP`; GET da tela, cujo
   `iframe#ifrArvore` tem de trazer `id_documento` igual a esse id e um
   `id_procedimento`; GET da árvore, conferida pelo NUP do nó `PROCESSO`
   e pelo id do processo, e o `DOCUMENTO` de mesmo id com o número SEI
   **igual** ao pedido (opaco, zeros à esquerda inclusos); GET do
   `Nos[i].src` assinado (`documento_visualizar` do mesmo id), cujo
   título `SEI/… - DOC - tipo` tem de ser do número pedido. Custo: 4 com
   o `action` guardado, 5 sem ele.

Pasta fechada (`AGUARDE`) só é aberta se o número pedido não estiver à
vista: um GET de "Abrir todas as Pastas" publicado na árvore, como em
[`processo NUP`](processo-obter-pastas.md) (+1). O aceite da forma com
documento é registrado por instalação, à parte da forma só com o
processo (medida na UnB; a recusa característica derruba a sessão de
todos e para o comando). Nenhuma outra URL sem assinatura é montada:
`documento_visualizar`, `documento_imprimir_web` e afins sem assinatura
derrubam a sessão de todos. O índice nunca causa requisição só para
aprendê-lo; falha do banco nunca derruba o comando.

## Saída

JSON `{"documentos": [...], "requisicoes": N}`; cada documento traz
`numero`, `id_documento`, `rotulo` (a5), `tipo` (do título), `icone`,
`cancelado`, `pasta`, `processo` (`nup`, `id`, `tipo`), `unidade` (da
tela; `null` pelo link da árvore, que não a publica), `resolvido_por`
(`indice_link`, `indice_trabalhar`, `indice_entrada_documento`,
`pesquisa_rapida` e, com `--processo`, também `indice_entrada` e
`caixa`), `requisicoes` (inclusive as dos palpites desmentidos),
`fonte: "documento_visualizar"`, os campos de versão do
[contrato de versões](documento-obter-versoes.md) e `texto` (o texto
visível do corpo, com as entidades HTML decodificadas). Sem `--json`, uma
linha por documento e o texto. Nunca URL, `infra_hash`, cookie ou HTML.

Com `--saida DIR` (diretório existente, conferido antes da rede), cada
interno grava só a foto `DOC.vN.html`, o HTML servido da versão lida,
0600, sem seguir symlink e de forma idempotente (mesmo SHA-256: nada
gravado, saída 0), **depois** de todos lidos; o JSON troca `texto` pelo
objeto `arquivo` (`nome`, `bytes`, `sha256`, `gravado`, `motivo`).
Nenhum `DOC.txt` nem `DOC.vN.txt` (substituídos em #207;
[versões e foto](documento-obter-versoes.md)).

## Falhas tipadas

Sem `--ocultar-inacessiveis`, o primeiro documento que falha encerra
o comando (saída 1, nada publicado), com `documento` e
`documentos_lidos_antes` no JSON. Com a opção, apenas
`ConteudoNaoPublicadoNaArvore` é omitido: saída 0 lista os documentos
legíveis; `documentos_omitidos` informa somente a contagem (sem
identificar os omitidos), e `requisicoes` inclui também a contagem
observada até cada omissão. Se todos forem omitidos, `documentos: []`.
Cancelamento, sessão encerrada, falha HTTP/transporte e divergência
de identidade continuam erros. Sem GET adicional ou retry.

| Erro (stderr) | Fato |
| --- | --- |
| `UsoInvalido::NumeroSeiRepetido` (saída 2) | o mesmo número duas vezes no pedido |
| `NumeroProcessoInvalido::Vazio` / `NumeroProcessoInvalido::Controle` / `NumeroProcessoInvalido::ReferenciaLegada` (saída 2) | `--processo` inválido, como em `processo NUP` |
| `ErroDocumentoObter::NumeroSeiAusenteDaArvoreDoProcesso` | `--processo`: a árvore do processo (pastas abertas) não tem o número |
| `ErroProcessoObter::EntradaPorIdRecusada` | a entrada com documento foi recusada ("Link sem assinatura"): a sessão morreu; a recusa fica no índice |
| `IdDocumentoDoRedirecionamentoDiferenteDoPedido` | interno: a entrada com documento endereçou outro id; o palpite é esquecido e o comando segue pela pesquisa |
| `PesquisaRapidaNaoAbriuODocumento` | a pesquisa não respondeu o 302 (visto com número inexistente; o resultado não é lido) |
| `EnderecoDaArvoreDeOutroDocumento` | a tela não seleciona o documento pesquisado (o número era de um processo) |
| `NumeroSeiDaArvoreDiferenteDoPedido` | o nó tem outro a15 (por exemplo `11505` pedido, `0011505` na árvore) |
| `ErroDocumentoObter::NumeroSeiRepetidoNaArvore` | dois `DOCUMENTO` com o número pedido |
| `ErroDocumentoObter::DocumentoAusenteDaArvore` | o id não está na árvore, nem com as pastas abertas |
| `ConteudoNaoPublicadoNaArvore` | `.src` vazio: a árvore não publica o conteúdo à sessão (visto com documento não assinado de outra unidade) |
| `PropriedadesNaoPublicadasNaBarra` | `--com-propriedades` e a barra do documento não publica o cadastro (sem GET) |
| `ErroDocumentoObter::DocumentoCancelado` | `.src` vazio num nó cancelado |
| `ErroDocumentoObter::Conteudo(ErroConteudoNaArvore::{LinkDoConteudoAusenteNaArvore, LinkDoConteudoRepetidoNaArvore, LinkDoConteudoForaDaFormaObservada, EnderecoDoConteudoNaoReconhecido, EnderecoDoConteudoDeOutroDocumento})` | `.src` ausente, repetido, fora da forma ou de outro documento |
| `ErroDocumentoObter::TituloDeOutroDocumento` | o HTML servido é de outro número SEI |
| `ErroArquivoTexto::DiretorioAusente` | `--saida`: o diretório falta (antes da rede, 0 requisições; link no caminho é aceito e o modo do `DIR` não é conferido) |
| `ErroCopia::FotoDivergente`, `ErroCopia::GravacaoFalhou` e os da foto | `--saida`: `DOC.vN.html` já existe com outro conteúdo, ou a gravação falhou ([versões e foto](documento-obter-versoes.md)) |

Sessão, Controle, formulário, 302, tela, árvore e pastas usam os
subcódigos de [`processo NUP`](processo-obter-arvore.md), inclusive
`SessaoEncerrada`.

## Efeitos

Leitura. A pesquisa rápida pode ficar lembrada na sessão; o GET da
árvore pode marcar o processo como visualizado (medido em GO no #59);
abrir as pastas pode ficar lembrado (como em `processo NUP`).
Se o GET do conteúdo marca o documento como visitado não foi medido. O
IES não restaura a tela.

Validação live: **PASS** na UnB e, em 04/11/2026, na UFRJ 4.1.5, na PEN3 e
na PEN4 5.0.4 (pesquisa rápida, link da árvore, `--processo`, `--saida`;
[varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)).

## Cache privado automático (#236)

A leitura Web grava automaticamente a representação integral em
`${XDG_CACHE_HOME:-$HOME/.cache}/ies/<host>/<DOC>/v<N>-<sha256[:16]>.<ext>`;
sem versão observada, usa `x-<sha256[:16]>.<ext>`. Documento e
instalação são a identidade, não o processo. Internos usam HTML servido;
externos usam PDF, TXT ou HTML original em fluxo, com limite e hash
já conferidos. A leitura não consulta lista apenas para nomear o cache.

`--sem-cache` e `IES_CACHE_DOCUMENTOS=0` desligam a escrita.
`--saida DIR` continua como cópia opcional. Ao copiar PDF do cache,
a foto recebe inode próprio, sem hard link: uma edição local não altera
o cache. Uma cópia já igual é no-op; uma divergente não é sobrescrita.
O JSON acrescenta `cache: {caminho, versao, sha256}` ou `null` e
`crc_rodape` do bloco assinado, quando observado, ou `null`.
CRC não é identidade ou validação criptográfica.

As gravações do cache usam diretórios privados 0700, arquivo 0600
e nome com SHA-256. Conteúdo com `infra_hash` é recusado; arquivos
preexistentes têm seu conteúdo confrontado com hash completo.

| Erro (stderr) | Fato |
| --- | --- |
| `FalhaCache::Diretorio` | caminho ou identidade do cache inválido |
| `FalhaCache::Gravacao` | não foi possível gravar ou publicar cache privado |
| `FalhaCache::ArquivoDivergente` | cópia existente diverge do conteúdo servido ou do SHA-256 integral |
