---
type: Contract
title: Detalhes de bloco por leitura Web
description: Contrato inicial de ies bloco N e tabela de documentos e processos
---

# ies bloco N

`ies bloco N [--json]`, N decimal positivo; inicialmente um bloco de
**assinatura**. O comando observa o Controle, segue o link da lista dos
blocos de assinatura e somente então segue o link de detalhe do número
exato publicado pela linha. A ação `rel_bloco_protocolo_listar` e
`id_bloco=N` devem aparecer uma única vez; URL resolvida na instalação,
`infra_hash` hexadecimal único, sem outro host, fragmento ou JS.

Orçamento: **3 GET, zero POST**, sem retry, sem redirect seguido.
O estado em `bloco.estado` provém da linha exata da lista que ofereceu o link,
não é inferido a partir dos itens do detalhe. O detalhe apresenta
`bloco.numero`, `bloco.tipo`,
`total_declarado`, `faixa_observada` e os `itens` da
`tblProtocolosBlocos`. Os campos Processo e Anotações devem existir;
Documento é exigido em blocos de assinatura. Campo ausente observado
como não disponível é null; não inferir documento para bloco interno.
A coluna é selecionada por cabeçalho, não por índice fixo.

Cabeçalho desconhecido, lista incompleta e detalhe com legenda
incoerente falham fechados, com subcódigo próprio. Se N não estiver
na página atual, `bloco_nao_encontrado_na_pagina` **não significa**
que o bloco não exista nas demais páginas.

**WIP:** consulta tipada de blocos internos/de reunião pelo número,
paginação do detalhe e listas vazias sem tabela ainda aguardam
replay e observação. Não contornar tais lacunas com HTML presumido.
Referência de cenários: Todos `blocos.ler`, `blocos_lista.py`,
`tests/paginas_reais/bloco_com_documentos.html` @323eb01;
cassettes `bloco-obter-concluido-*`.
