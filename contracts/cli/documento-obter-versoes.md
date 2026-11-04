---
type: Contract
title: Ler versão de documento e preparar cópia de trabalho
description: ies documento DOC --versao N lê a versão N pela lista de versões que a barra do documento publica na árvore; versao_atual só sai quando a lista foi lida; com --saida, a leitura prepara DOC.vN.html quando a árvore declara o documento editável e não assinado
---

# Versões e cópia de trabalho de `ies documento DOC` (Refs #88)

Complementa a [leitura Web](documento-obter-web.md). Os passos 1 a 4
(pesquisa rápida, 302, tela e árvore conferida) são os mesmos; este
contrato trata do que vem depois da árvore.

```text
ies documento DOC [DOC ...] [--versao N] [--saida DIR] [--sem-copia] [--json] [--unidade SIGLA]
```

## Onde a lista está publicada

A barra do nó `DOCUMENTO` (`Nos[i].acoes`, o literal inteiro de uma
linha) publica o link assinado
`controlador.php?acao=documento_versao_listar&…&id_procedimento=P&id_documento=D&…&infra_hash=…`
("Versões do Documento"). Medido no acervo público: as sete instalações
(UnB, UFRJ, GO, PEN1 a PEN4) gravaram a lista seguindo esse link da
barra. O link sem assinatura derruba a sessão (ADR 0003): o IES só segue
o link da barra, conferido (`acao` única, `id_documento` igual ao do nó,
`id_procedimento` igual ao do processo, `infra_hash` de 64 hex). Documento
externo, de outra unidade ou sem permissão não traz o link.

A mesma barra declara a edição: `onclick="editarConteudo('N');"` (o JS
da árvore trata `'N'` como não assinado na montagem da tela) ou
`editarConteudo('S')` (assinado: editar perde a assinatura). Ausente,
o conteúdo não é editável pela sessão. O IES não segue nenhum desses
links e nunca abre o editor (`editor_montar`).

A tela da lista (`table.infraTable` com legenda única
`Lista de Versões (N registros):`) traz, por linha, versão, usuário,
unidade, data e, em Ações, **um** link `documento_visualizar` assinado
com `versao=N`, o `id_documento` do nó (servido duas vezes, com o mesmo
valor, nas sete instalações) e o `id_procedimento`. Outras âncoras da
célula (`href="#ID-N"` do "Recuperar Versão", cujo link fica no
`onclick`) não são seguidas. Total declarado diferente das linhas lidas
é recusa.

## Caminhos e requisições

Sem retry e sem seguir redirect. `G` = 4 com o `action` guardado, 5 sem
ele (passos 1 a 4 da leitura Web).

| Pedido | Requisições | O que se lê |
| --- | --- | --- |
| sem `--versao`, sem `--saida` (ou com `--sem-copia`) | G + 1 | o conteúdo atual (`Nos[i].src`); a lista não é lida |
| `--saida DIR` sem `--sem-copia`, barra com lista e `editarConteudo('N')` | G + 2 | conteúdo atual e a lista (para saber N) |
| `--saida DIR` sem `--sem-copia`, barra sem lista ou sem edição não assinada | G + 1 | só o conteúdo; cópia não preparada, com motivo |
| `--versao N` | G + 2 | a lista e o `documento_visualizar` da linha N; o conteúdo atual **não** é pedido |

Decisão de custo (proprietário, 04/11/2026): `versao_atual` custa 1 GET
e só é publicada quando a leitura já pediu a lista; nenhuma tela lida
nos passos 1 a 5 traz o número da versão.

## Saída

Cada documento do JSON da leitura Web ganha:

- `versao`: N lido com `--versao N`, senão `null`;
- `versao_atual`: a maior versão da lista, quando lida, senão `null`;
- `versao_atual_motivo`: `null` quando publicada; senão `nao_pedida`,
  `lista_de_versoes_nao_publicada_na_arvore` ou `copia_nao_preparavel`
  (a cópia não era possível, então a lista não foi pedida);
- `copia`: `{"arquivo": "DOC.vN.html", "bytes": B}` ou `null`;
- `copia_motivo`: `null` quando gravada; senão `nao_pedida` (sem
  `--saida` ou com `--sem-copia`), `lista_de_versoes_nao_publicada_na_arvore`,
  `documento_assinado`, `edicao_nao_publicada_na_arvore`,
  `versao_historica` (N menor que a atual) ou `conteudo_com_link_assinado`.

`texto` (ou `arquivo` com `--saida`) é o da versão lida. Nunca URL,
`infra_hash`, cookie ou HTML no terminal.

## Cópia de trabalho

Com `--saida DIR` e sem `--sem-copia`, depois de todos os documentos
lidos e do `DOC.txt`, a leitura grava `DOC.vN.html` (o HTML servido do
conteúdo, em UTF-8), a base `.DOC.vN.base.html` e o manifesto
`.DOC.vN.meta.json` (documento, id, instalação, versão, SHA-256 da base),
0600, sem sobrescrever nem seguir symlink — o formato de
[`documento salvar`](documento-salvar.md). Só quando seguro: a barra
publica a lista **e** `editarConteudo('N')`, a versão lida é a atual e o
HTML não contém `infra_hash`. O `'N'` é a declaração da árvore na
montagem; o próprio SEI ainda confere a assinatura por AJAX antes de
editar, e o `salvar` remoto (#89) terá de conferir de novo.

## Falhas tipadas

| Subcódigo | Fato |
| --- | --- |
| `versao_documento_invalida` | `--versao 0` (antes de qualquer rede) |
| `lista_de_versoes_nao_publicada_na_arvore` | `--versao N` e a barra do documento não publica a lista |
| `versao_nao_observada` | N não está entre as linhas da lista |
| `link_das_versoes_*`, `barra_do_documento_*` | barra repetida, fora da forma, link repetido, de outro documento ou processo |
| `historico_documento_ausente`, `lista_versoes_incompleta`, `colunas_historico_inesperadas`, `linha_historico_incompleta`, `link_versao_ausente_ou_ambiguo`, `link_versao_inseguro`, `identidade_documento_divergente`, `versao_documento_duplicada`, `legendas_historico_versoes_repetidas` | a lista servida fora da forma observada |
| `titulo_do_conteudo_de_outro_documento` | a versão servida é de outro número SEI |
| `documento_externo_por_download_de_anexo`, `documento_cancelado_sem_conteudo_na_arvore`, `conteudo_nao_publicado_na_arvore` | como na leitura Web, antes da lista |
| `copia_versao_ja_existe`, `destino_copia_inseguro`, `copia_local_link_simbolico`, `gravacao_copia_falhou` | a cópia não foi gravada (nada publicado) |

## Efeitos

Leitura. A lista e a versão são GET de telas publicadas; se marcam algo
como visitado não foi medido. Nenhuma escrita, nenhum editor, nenhuma
recuperação de versão.
