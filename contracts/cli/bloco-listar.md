---
type: CommandContract
operation: bloco.listar
implementation: partial
title: Leitura Web de blocos da unidade
description: Contrato inicial de ies bloco listar por tipo e página observada
---

# ies bloco listar

Comando: `ies bloco listar [--tipo assinatura|interno|reuniao] [--pagina N] [--json]`.
`--tipo` omitido usa `assinatura`. `--unidade` e `--json` são globais.

A origem é estritamente o SEI Web: GET do Controle com origem e cookie
validados, GET do link exato `bloco_assinatura_listar`,
`bloco_interno_listar` ou `bloco_reuniao_listar` publicado no mesmo
Controle. Não há fallback REST/SOAP nem URL de ação presumida.

Orçamento da leitura de uma página: **2 GET, zero POST**, sem retry, sem
seguir redirect, sem restaurar página/filtro ou executar JS. Falhas anteriores
ao primeiro GET consomem zero pedidos. A saída declara o número de GETs
realmente feitos apenas nos sucessos. Sessão ativa é pré-requisito.

O modelo da lista publica `tipo`, `total_declarado`,
`faixa_observada` e `itens`. Cada item contém `numero`, `tipo`,
`estado`, `descricao` e `quantidade_itens`, com **null** quando a
quantidade não foi mostrada pela tela de lista. Não deduz número de documentos
da quantidade de blocos. Link assinado, cookie e id interno jamais aparecem
na saída.

A tabela `tblBlocos` é lida pelos títulos de coluna, não pela posição
fixa, e exige Número, Estado, Descrição, legenda e número de linha coerentes.
Cabeçalho truncado, número repetido, tabela repetida e faixa contraditória
são erros tipados, nunca listas parciais declaradas completas.

**WIP:** A paginação N>1 ainda está bloqueada antes de qualquer rede com
`pagina_de_blocos_nao_implementada`; `--pagina 1` confere se a página
observada começa na primeira. A leitura sem `--pagina` publica apenas a
página que a sessão atualmente apresenta. Nunca apresenta a primeira página
como se fosse a lista inteira. O formulário `frmBlocoLista` e os filtros
devem ser observados, contratados e testados antes de habilitar POST de
paginação, sem restaurar a sessão.

Fonte de cenários: Todos `src/todos/sei/blocos_lista.py` e
`tests/paginas_reais/menu_blocos.html` @323eb01; cassettes
`bloco-lista-obter-buscar-*` e `bloco-interno-ciclo-*`.
A leitura ao vivo em UnB e GO permanece NOT_RUN até teste autorizado
com requisições medidas. Escrita ao vivo fora do escopo.
