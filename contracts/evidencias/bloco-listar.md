---
type: Guide
title: Evidências dos comandos Web de blocos
description: Casos derivados de Todos e limites de observação para bloco.listar
---

# Evidências — bloco.listar (#256)

Fonte: Todos @323eb01, `src/todos/sei/blocos_lista.py`,
`tests/paginas_reais/menu_blocos.html`,
`tests/paginas_reais/bloco_com_documentos.html` e
`tests/test_bloco_de_processos_010.py`. Esses arquivos são somente
oráculo de formato e de cenário, não arquitetura a portar.

Testes Rust sintéticos em `src/bloco.rs`, `src/bloco_detalhe.rs`
e `tests/bloco_listar.rs`, sem credenciais em fixtures.
O cenário feliz lê Controle + lista = **2 GET**; obter detalhe consome
mais um GET = **3 GET**. Nenhum envio POST. A primeira página não
é inventada quando a sessão está em outra página.

UnB e Goiás: **NOT_RUN**, sem requisições reais; nenhuma alteração no
SEI. Replay das cassettes `bloco-lista-obter-buscar-*` e
`bloco-interno-ciclo-*`: NOT_RUN. RED local: NOT_RUN por falta
de Rust em sandbox; gates do CI são a fonte de confirmação de testes.
