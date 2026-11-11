---
type: Contract
title: Detalhes de bloco por leitura Web
description: Contrato inicial de ies bloco N e tabela de documentos e processos
---

# ies bloco N

`ies bloco N [--json]`, N decimal positivo, consulta o bloco de
**assinatura, interno ou reunião**. O comando observa o Controle e visita
as listas tipadas que ele efetivamente publica, sem escolher assinatura
por padrão. Resolve o número somente quando uma linha o identifica e
as demais listas permitem excluir colisões: se aparecer em mais de um
tipo, ou se uma lista sem a linha estiver incompleta, a identidade não
está comprovada e a CLI recusa (saída 1). A ação
`rel_bloco_protocolo_listar` e `id_bloco=N` devem aparecer uma única vez
no link da linha encontrada; URL resolvida na instalação, `infra_hash`
hexadecimal único, sem outro host, fragmento ou JS.

Orçamento de leitura com identidade inequívoca: **1 GET de Controle +
1 GET por lista tipada oferecida + 1 GET de detalhe, zero POST**, sem
retry ou redirect seguido. A ausência numa página parcial não prova
ausência do bloco em todo o acervo.
O estado em `bloco.estado` provém da linha exata da lista que ofereceu o link,
não é inferido a partir dos itens do detalhe. O detalhe apresenta
`bloco.numero`, `bloco.tipo`,
`total_declarado`, `faixa_observada` e os `itens` da
`tblProtocolosBlocos`. Os campos Processo e Anotações devem existir;
Documento é exigido em blocos de assinatura. Para blocos internos e de
reunião, o detalhe representa processos, sem as chaves `documento` e
`tipo` quando essas colunas não foram publicadas (nunca `null`, #749);
não inferir documento a partir de
uma célula vazia.
A coluna é selecionada por cabeçalho, não por índice fixo.

Cabeçalho desconhecido, lista incompleta e detalhe com legenda
incoerente falham fechados, com subcódigo próprio. Se N não estiver
na página atual, `bloco_nao_encontrado_na_pagina` **não significa**
que o bloco não exista nas demais páginas.

**WIP:** paginação do detalhe e variações de listas sem tabela ainda
aguardam replay e observação. A ampliação tipada aqui é sintética até
confronto bilateral com a cassette PEN1 e validação em treinamento.
Não contornar lacunas com HTML presumido.
Referência de cenários: Todos `blocos.ler`, `blocos_lista.py`,
`tests/paginas_reais/bloco_com_documentos.html` @323eb01;
cassettes `bloco-obter-concluido-*`.
