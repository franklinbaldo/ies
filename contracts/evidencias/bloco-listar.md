---
type: OperationEvidence
operation: bloco.listar
source: todos-demonstration-matrix
source_commit: 323eb01
unresolved_steps: 3
title: Evidências dos comandos Web de blocos
description: Casos derivados de Todos e limites de observação para bloco.listar
---

# Evidências — bloco.listar (#256)

Fonte: Todos @323eb01, `src/todos/sei/blocos_lista.py`,
`tests/paginas_reais/menu_blocos.html`,
`tests/paginas_reais/bloco_com_documentos.html` e
`tests/test_bloco_de_processos_010.py`. Esses arquivos são somente
oráculo de formato e de cenário, não arquitetura a portar.

Testes Rust sintéticos em `src/bloco.rs`, `src/bloco_detalhe.rs`,
`tests/bloco_listar.rs` e `tests/aies_bloco_cli.rs`, sem credenciais em
fixtures. A CLI real foi exercitada no último teste por HTTPS local contra
anti-IES com SQLite efêmero. A primeira página lê Controle + lista =
**2 GET, zero POST**; obter detalhe consome mais um GET = **3 GET**.
A **segunda página interna** deriva **um POST único de navegação** de
`frmBlocoLista`, após os 2 GET, e exige mesmo total e faixa contígua;
outras listas na página 2 e páginas acima de 2 continuam recusadas antes
da rede. Não restaurar página/filtro da sessão. A primeira página não
é inventada quando a sessão está em outra página.

O card Goiás `bloco-interno-ciclo-go` registra uma transição de faixa
1–50 para 51–100 após POST de navegação, mas **não preserva o corpo
original do POST**. Os pares derivados pela CLI são hipótese sintética,
não equivalência externa demonstrada. No SQLite, página desconhecida
não vira página vazia; `total_declarado=0`, sem a chave `faixa_observada` e
`itens=[]` só são publicados quando o vazio foi explicitamente registrado.

**Três evidências distintas:** testes sintéticos CLI real ↔ anti-IES ↔
SQLite estão presentes em `tests/aies_bloco_cli.rs` (BDD-256-01/02/03),
mas não foram reexecutados nesta revisão documental. O confronto
independente entre leitores IES, anti-IES e corpos originais das cassettes
`bloco-lista-obter-buscar-*` e `bloco-interno-ciclo-*` é **NOT_RUN**.
UnB e Goiás ao vivo: **NOT_RUN**, sem requisições externas nem alteração
no SEI por esta revisão. Os cards e seus hashes publicados não são, por
si só, prova de replay semântico.

PENDENTE_DE_CASSETTE: confronto bilateral de cassettes, paginação além da segunda
página interna e observação em treinamento.
