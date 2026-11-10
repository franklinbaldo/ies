---
type: OperationEvidence
title: processo.obter
operation: processo.obter
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: false
ies_implemented: true
upstream_steps: 40
unresolved_steps: 0
---

# processo.obter

Origem: matriz de demonstrações do Todos. IES Rust: PARCIAL WEB na
leitura pela árvore (`--resumo` e as opções de captura privada foram
aposentados em 04/11/2026)
([contrato](../cli/processo-obter-arvore.md), Refs #59, fatia 1) e dos
andamentos ([contrato](../cli/processo-obter-andamentos.md), fatia 2) e
das pastas ([contrato](../cli/processo-obter-pastas.md), fatia 3).
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

## Evidência própria do IES (fatia 1)

- Pesquisa rápida: nas cassettes `processo-andamento-go` e
  `processo-nup-inexistente-go`, o POST de `protocolo_pesquisa_rapida`
  com NUP existente respondeu 302 para `procedimento_trabalhar` com
  `acao_origem=protocolo_pesquisa_rapida` e `id_protocolo`; o inexistente,
  200 com a tela de pesquisa. Seguem-se `procedimento_trabalhar` (200) e
  `procedimento_visualizar` (200).
- Capturas privadas `0600`, lidas só pela estrutura (03/11/2026): tela do
  processo e árvore de UnB e GO passaram pelos leitores; o formulário
  rápido e o link da linha do Controle de UnB, GO, PEN1, PEN3, PEN4 e
  UFRJ também.
- Medição do agente pai (03/11/2026): pesquisa rápida com o NUP exato
  abriu o processo na UnB, em GO e na PEN1, inclusive NUPs que a pesquisa
  avançada não achou em GO e na PEN1.
- Validação live do comando IES: **PASS** na UnB, em GO e na PEN1 ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-arvore.md)).
- Replay offline das 495 cassettes públicas pelos mesmos leitores (tela,
  árvore, pastas, histórico, propriedades, marcadores, situação e
  natureza), lido/recusado/não aplicável por instalação, achados abertos
  ([relatório](../../docs/validacao-replay/2026-11-04-processo-obter-acervo.md)).

## Evidência própria do IES (fatia 2, andamentos)

- Capturas privadas `0600` (03/11/2026), só estrutura: árvore da UnB
  (link na função `consultarAndamento`) e de GO (link no `onclick`),
  histórico `R` da UnB e `P` de GO (49 e 17 linhas) passaram pelo leitor
  do link, pelo leitor do histórico (`src/historico.rs`), pela legenda e pela regra
  de `verHistorico`; o POST `P` derivou pelo primitivo de formulário.
- Medições do agente pai: visão `P` inteira até 75 linhas em 14 processos
  de GO, sem paginador; duas árvores de GO sem o link; um 503 no
  histórico; 403 do WAF de GO depois de rajadas; ablação de
  `hdnInfraNroItens`/`hdnInfraPaginaAtual` na UnB sem efeito.
- [Contrato](../cli/processo-obter-andamentos.md) e
  [observação](../../docs/como-o-sei-funciona/historico-do-processo.md).
  Validação live do comando: **PASS** na UnB, na PEN1 e em GO ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-andamentos.md)).

## Evidência própria do IES (fatia 3, pastas)

- `processo-em-pastas-unb.json` (ZIP da UnB, SHA-1
  `21e0559015c56c403daf8933e5040504a7b83837`; cassette SHA-256
  `0bc73a70deacb3af4b16edc0851768580fa3046ac3f676bc56a1fe8f0de5a369`):
  interação 8, árvore com 8 `PASTA`, 7 `AGUARDE` e 17 documentos;
  interação 9, GET de `ABRIR_PASTAS`, 8 pastas e 157 documentos, sem
  `AGUARDE`. Os `id_rel_protocolo_protocolo` das barras igualam os
  `Pastas[k]['protocolos']` (17/17 e 157/157).
- `processo-pasta-mista-unb.json` (lote SHA-1
  `4838ecae4d5a132a60d38e5783171a8640a36b30`; cassette SHA-256
  `dffe2d9379ebcaccd6df56d188b4ae86c93ed3e9fbfd02f721cbdcbaabc7dd42`):
  20 documentos sem pasta; com 21, duas pastas (uma fechada de 20, uma
  aberta); interações 412, 418, 433, 439, 462 e 466 abrem tudo por GET; a
  árvore simples seguinte volta fechada. Nenhum POST de
  `procedimento_paginar`, nenhum `FECHAR_PASTAS`.
- Medição ao vivo do agente pai na UnB (03/11/2026): dois processos com
  pastas (157 documentos em 8 pastas e um de 2 pastas) abriram inteiros
  por `ABRIR_PASTAS` (nenhum `AGUARDE`); as contagens por pasta igualaram
  `Pastas[k]['protocolos']` (7×20 + 17), mas 150 barras vieram vazias
  (processo aberto só em outra unidade). Na revalidação, a identidade
  variou por documento: no processo de 8 pastas, PASTA1–5 e 7 só com
  barras vazias, PASTA6 com 18 vazias e 2 com links sem
  `id_rel_protocolo_protocolo`, PASTA8 com 12 e 5; no de 2 pastas, PASTA1
  com 19 identidades e 1 vazia, PASTA2 com 2. O leitor deu
  `por_contagem` e `mista`, nessa ordem.
- [Contrato das pastas](../cli/processo-obter-pastas.md). Validação live do
  comando: **PASS** na UnB ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-pastas.md)).

## Evidência própria do IES (anexados)

- Captura privada ao vivo da UnB 4.0.9 (03/11/2026, `0600`, lida só pela
  estrutura): a árvore de um processo com 9 nós, o `PROCESSO`, um
  `PROCESSO_ANEXADO` e 7 `DOCUMENTO`, todos com 16 argumentos. O anexado
  tem a2 igual ao id do processo, a3 `arvore_visualizar` com
  `id_procedimento` do processo e `id_procedimento_anexado` igual a a1,
  a12–a14 `null`, a15 igual a a5 e barra própria (`procedimento_alterar`,
  `comentario_listar`). Os 7 documentos são filhos do processo; nenhum nó
  tem o anexado como pai. Antes desta fatia o comando recusava a árvore
  com `TipoDeNoDaArvoreNaoSuportado`; o leitor novo aceitou a
  captura (teste local temporário, apagado).
- Captura privada da árvore do próprio anexado (mesmo dia, aberta pela
  pesquisa rápida do NUP dele): 2 nós, raiz `PROCESSO` com id e NUP iguais
  aos do nó `PROCESSO_ANEXADO` da árvore do pai e ícones
  `processo_anexado.svg`; o `.html` da raiz tem um único `<a>`, `href`
  `procedimento_trabalhar` com `id_procedimento` igual ao id do pai,
  `id_procedimento_anexado` igual ao da raiz, hash de 64 hex e
  `infra_sistema`/`infra_unidade_atual` iguais aos do a3 da raiz; texto
  igual ao NUP do pai, sem escape nem entidade.
- Validação live do comando com anexado: **PASS** na UnB ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-anexados.md)).

## Evidência própria do IES (documentos movidos)

- Captura privada ao vivo da PEN1 5.0.4 (04/11/2026, `0600`,
  `iso-8859-1`, lida só pela estrutura): árvore de 4 nós de 16
  argumentos, 1 `PROCESSO`, 2 `DOCUMENTO` e 1 `DOCUMENTO_MOVIDO` com a1
  `<n>-<m>`, a2 igual ao id do processo, a3 igual ao do nó `PROCESSO` e
  a15 igual ao número entre parênteses de a5. O `.html` do movido tem um
  único `<a>` `procedimento_trabalhar` (hash de 64 hex) com
  `id_procedimento` diferente do processo e `id_documento` igual a `n`;
  texto igual ao NUP da dica. A barra só tem `comentario_listar`, com
  `id_rel_protocolo_protocolo` igual a `m`. Nenhum `DOCUMENTO` tem id `n`.
  Antes desta fatia o comando recusava a árvore com
  `TipoDeNoDaArvoreNaoSuportado`; o leitor novo aceitou a captura
  (2 documentos, 1 movido com destino conferido, sem vínculo de anexação;
  teste local temporário, apagado).
- [Contrato](../cli/processo-obter-arvore.md) e
  [observação](../../docs/como-o-sei-funciona/arvore-do-processo.md).
  Validação live: **NOT_RUN**.
- Cassettes públicas `documento-mover-cancelar-{go,pen1,pen2,pen3}`
  (replay offline): documento movido de A para B e de volta; as 8
  árvores seguintes de A trazem o movido `n-m` e o `DOCUMENTO` `n`.
  [Contrato dos movidos](../cli/processo-obter-movidos.md),
  [relatório](../../docs/validacao-replay/2026-11-04-processo-obter-acervo.md).

## Evidência própria do IES (propriedades)

- Captura privada ao vivo da UnB 4.0.9 (04/11/2026, `0600`, `iso-8859-1`,
  lida só pela estrutura): `GET procedimento_alterar` da barra respondeu
  200 com `form#frmProcedimentoCadastro`; `hdnProtocoloFormatado` e
  `hdnIdProcedimento` dão a identidade; tipo em `option selected` com
  `hdnIdTipoProcedimento` igual; 1 assunto em `selAssuntos` (texto
  `<código> - <descrição>`), 0 interessados, `txaObservacoes` vazio,
  `optPublico` marcado, `selHipoteseLegal` sem opções, `hdnDtaGeracao`
  `dd/mm/aaaa`; `hdnAssuntos` e `hdnInteressadosProcedimento` vazios. O
  leitor aceitou a tela e as seis árvores capturadas (quatro com
  `procedimento_alterar`, duas com `procedimento_consultar`) em teste
  local temporário, apagado.
- [Contrato](../cli/processo-obter-propriedades.md) e
  [observação](../../docs/como-o-sei-funciona/propriedades-do-processo.md).
  Validação live: **NOT_RUN**.

## Evidência própria do IES (marcadores)

- Capturas privadas ao vivo da UnB 4.0.9 (04/11/2026, `0600`,
  `iso-8859-1`, lidas só pela estrutura): a barra do nó `PROCESSO` das
  três árvores completas publica `andamento_marcador_gerenciar`; o GET
  respondeu 200 com `<title>SEI - Marcadores do Processo <NUP>`,
  `form#frmGerenciarMarcador`, `btnAdicionar`/`btnRemover` (escrita, não
  tocada) e `table#tblMarcadores` com legenda `(2 registros)` e duas
  linhas (checkbox com o id, ícone `svg/marcador_<cor>.svg`, nome, texto
  vazio numa e preenchido na outra, sigla em `a.ancoraSigla`, data/hora).
  A célula Marcadores da visão detalhada do mesmo processo tem duas
  `a.ancMarcador` com `#ID-<n>` iguais aos ids da tela. Os dois leitores
  deram dois marcadores em teste local temporário, apagado.
- [Contrato](../cli/processo-obter-marcadores.md),
  [observação](../../docs/como-o-sei-funciona/marcadores-do-processo.md)
  e [decisão](../../docs/decisoes/processo-obter/marcadores.md).
  Validação live: **NOT_RUN**.

## Evidência própria do IES (situação e natureza)

- Capturas privadas das árvores da UnB 4.0.9 (seis), de GO 5.0.3 (uma) e
  da PEN1 5.0.4 (uma), `0600`, `iso-8859-1`, lidas só pela estrutura: o
  `.html` do nó `PROCESSO` em seis formas (somente na unidade, com e sem
  atribuição; nas unidades, com e sem atribuição; sem andamentos abertos;
  aviso de anexação) e sete radicais de ícone de documento, um cancelado
  (dica `Documento Cancelado: <motivo>`). Em teste local temporário
  (apagado), o leitor deu as unidades esperadas nas sete árvores não
  anexadas, "fora da forma" no anexado e `cancelado: true` só no
  cancelado.
- [Contrato](../cli/processo-obter-situacao.md),
  [observação](../../docs/como-o-sei-funciona/situacao-na-arvore.md) e
  [decisão](../../docs/decisoes/processo-obter/situacao.md).
  Validação live do comando: **PASS** na UnB ([relatório](../../docs/validacao-live/2026-11-04-processo-obter-situacao.md)).

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-unb#8` → [marcador-escritas-unb--d380f9b1bd77.md](../../cassettes/unb/marcador-escritas-unb--d380f9b1bd77.md)
- `processo-andamento-completo-unb#1` → [processo-andamento-completo-unb--ddd49eff108e.md](../../cassettes/unb/processo-andamento-completo-unb--ddd49eff108e.md)
- `processo-andamento-resumido-unb#1` → [processo-andamento-resumido-unb--e06a217b9d6e.md](../../cassettes/unb/processo-andamento-resumido-unb--e06a217b9d6e.md)
- `processo-andamento-unb#1` → [processo-andamento-unb--7ad43ba8cdae.md](../../cassettes/unb/processo-andamento-unb--7ad43ba8cdae.md)
- `processo-em-pastas-unb#1` → [processo-em-pastas-unb--0bc73a70deac.md](../../cassettes/unb/processo-em-pastas-unb--0bc73a70deac.md)
- `processo-nup-inexistente-unb#2` → [processo-nup-inexistente-unb--602e2f52793c.md](../../cassettes/unb/processo-nup-inexistente-unb--602e2f52793c.md)
- `processo-pasta-mista-unb#46` → [processo-pasta-mista-unb--dffe2d9379eb.md](../../cassettes/unb/processo-pasta-mista-unb--dffe2d9379eb.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-pen1#8` → [marcador-escritas-pen1--7b1fc804189f.md](../../cassettes/pen1/marcador-escritas-pen1--7b1fc804189f.md)
- `processo-andamento-completo-pen1#1,2` → [processo-andamento-completo-pen1--2418e3a42844.md](../../cassettes/pen1/processo-andamento-completo-pen1--2418e3a42844.md)
- `processo-andamento-pen1#1` → [processo-andamento-pen1--883fd03d8c51.md](../../cassettes/pen1/processo-andamento-pen1--883fd03d8c51.md)
- `processo-criar-relacionado-pen1#5` → [processo-criar-relacionado-pen1--c8d663fd7a0c.md](../../cassettes/pen1/processo-criar-relacionado-pen1--c8d663fd7a0c.md)
- `processo-nup-inexistente-pen1#2` → [processo-nup-inexistente-pen1--1ee7f1b16451.md](../../cassettes/pen1/processo-nup-inexistente-pen1--1ee7f1b16451.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-pen2#8` → [marcador-escritas-pen2--50f1ece0ccfa.md](../../cassettes/pen2/marcador-escritas-pen2--50f1ece0ccfa.md)
- `processo-andamento-completo-pen2#1,2` → [processo-andamento-completo-pen2--cc1197c62434.md](../../cassettes/pen2/processo-andamento-completo-pen2--cc1197c62434.md)
- `processo-andamento-pen2#1` → [processo-andamento-pen2--0580f7d4c788.md](../../cassettes/pen2/processo-andamento-pen2--0580f7d4c788.md)
- `processo-nup-inexistente-pen2#2` → [processo-nup-inexistente-pen2--1d9e2290454a.md](../../cassettes/pen2/processo-nup-inexistente-pen2--1d9e2290454a.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `documento-consultar-fechado-ufrj#7` → [documento-consultar-fechado-ufrj--f018ca2b1c8f.md](../../cassettes/ufrj/documento-consultar-fechado-ufrj--f018ca2b1c8f.md)
- `marcador-escritas-ufrj#8` → [marcador-escritas-ufrj--93c12886b4dd.md](../../cassettes/ufrj/marcador-escritas-ufrj--93c12886b4dd.md)
- `processo-andamento-completo-ufrj#1,2` → [processo-andamento-completo-ufrj--6154b2b9d7c9.md](../../cassettes/ufrj/processo-andamento-completo-ufrj--6154b2b9d7c9.md)
- `processo-andamento-ufrj#1` → [processo-andamento-ufrj--6aa848bf537c.md](../../cassettes/ufrj/processo-andamento-ufrj--6aa848bf537c.md)
- `processo-nup-inexistente-ufrj#2` → [processo-nup-inexistente-ufrj--e88e670e57a8.md](../../cassettes/ufrj/processo-nup-inexistente-ufrj--e88e670e57a8.md)
- `processo-pasta-mista-ufrj#46` → [processo-pasta-mista-ufrj--65861ad4af07.md](../../cassettes/ufrj/processo-pasta-mista-ufrj--65861ad4af07.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-pen3#8` → [marcador-escritas-pen3--23493f6979fa.md](../../cassettes/pen3/marcador-escritas-pen3--23493f6979fa.md)
- `processo-andamento-completo-pen3#1,2` → [processo-andamento-completo-pen3--fb26b6cc0e8f.md](../../cassettes/pen3/processo-andamento-completo-pen3--fb26b6cc0e8f.md)
- `processo-andamento-pen3#1` → [processo-andamento-pen3--2997bd53122c.md](../../cassettes/pen3/processo-andamento-pen3--2997bd53122c.md)
- `processo-nup-inexistente-pen3#2` → [processo-nup-inexistente-pen3--3f9958e9e262.md](../../cassettes/pen3/processo-nup-inexistente-pen3--3f9958e9e262.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-pen4#8` → [marcador-escritas-pen4--584db9430706.md](../../cassettes/pen4/marcador-escritas-pen4--584db9430706.md)
- `processo-andamento-completo-pen4#1,2` → [processo-andamento-completo-pen4--083571aeb3aa.md](../../cassettes/pen4/processo-andamento-completo-pen4--083571aeb3aa.md)
- `processo-andamento-pen4#1` → [processo-andamento-pen4--4ecb417eb7a7.md](../../cassettes/pen4/processo-andamento-pen4--4ecb417eb7a7.md)
- `processo-nup-inexistente-pen4#2` → [processo-nup-inexistente-pen4--808885a93f1e.md](../../cassettes/pen4/processo-nup-inexistente-pen4--808885a93f1e.md)

## go

Estado registrado no Todos: **demonstrada**.

- `marcador-escritas-go#8` → [marcador-escritas-go--7c45b39d550f.md](../../cassettes/go/marcador-escritas-go--7c45b39d550f.md)
- `processo-andamento-completo-go#1,2` → [processo-andamento-completo-go--8b8110e1b484.md](../../cassettes/go/processo-andamento-completo-go--8b8110e1b484.md)
- `processo-andamento-go#1` → [processo-andamento-go--ae86a5f02b6f.md](../../cassettes/go/processo-andamento-go--ae86a5f02b6f.md)
- `processo-nup-inexistente-go#2` → [processo-nup-inexistente-go--cf432a7c366d.md](../../cassettes/go/processo-nup-inexistente-go--cf432a7c366d.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
