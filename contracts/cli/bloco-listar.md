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

Orçamento sem `--pagina` ou `--pagina 1`: **2 GET, zero POST**.
Para `--tipo interno --pagina 2`, a tela inicial autoriza **2 GET e um POST**
único do `frmBlocoLista` usando a opção `1` zero-based publicada. A resposta
precisa demonstrar a faixa contígua da segunda página e o mesmo total.
Sem retry, sem seguir redirect, sem restaurar página/filtro ou executar JS. Falhas anteriores
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

**Parcial:** o `--tipo interno --pagina 2` usa um único POST a partir da
primeira página e exige a faixa correspondente, com estado compartilhado no
anti-IES. Páginas acima de 2 e a página 2 dos outros tipos permanecem
bloqueadas antes da rede (`pagina_de_blocos_nao_implementada`).
`--pagina 1` confere a primeira faixa. Sem `--pagina`, a CLI publica
somente a faixa observada, nunca presume a lista inteira. A cassette Goiás
`bloco-interno-ciclo-go` registrou GET da faixa 1–50, seguido de POST
respondendo com 51–100 (`hdnInfraPaginaAtual=1`). Os **corpos originais dos
POSTs não foram preservados**, portanto os pares do formulário derivado são
hipótese que requer confronto e ensaio de treinamento. A mudança de página
altera o estado da sessão e não é revertida.

Fonte de cenários: Todos `src/todos/sei/blocos_lista.py` e
`tests/paginas_reais/menu_blocos.html` @323eb01; cassettes
`bloco-lista-obter-buscar-*` e `bloco-interno-ciclo-*`.
A leitura ao vivo em UnB e GO permanece NOT_RUN até teste autorizado
com requisições medidas. Escrita ao vivo fora do escopo.
