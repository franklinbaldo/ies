---
type: Guide
title: Matriz de derivação da CLI de blocos
description: Cenários de teste e evidências para os leitores de blocos
---

# Matriz — bloco (#256)

| Operação IES | Caso oráculo Todos @323eb01 | Evidência | Resultado |
| --- | --- | --- | --- |
| `bloco listar` | `tests/paginas_reais/menu_blocos.html` e `blocos_lista.py` | `tblBlocos`, cabeçalhos por nome, legenda e linha | cobertura sintética integrada; CI novo NOT_RUN |
| `bloco listar --tipo interno` | `docs/como-o-sei-funciona/15-telas-da-barra/blocos-internos.md` | ação `bloco_interno_listar` | código/teste sintético; live NOT_RUN |
| `bloco listar --tipo reuniao` | `blocos.py` / `bloco_reuniao_listar` | menu observado | código/teste sintético; live NOT_RUN |
| `bloco N` | `tests/paginas_reais/bloco_com_documentos.html` | `tblProtocolosBlocos` | código/teste sintético; replay NOT_RUN |
| Bloco interno | `tests/test_bloco_de_processos_010.py` | Processo sem Documento | teste sintético; live NOT_RUN |
| Falha fechada | `bloco-lista-obter-buscar-*` | número repetido, legenda truncada, URL assinada ambígua | teste sintético; replay NOT_RUN |

Orçamento da leitura de assinatura: lista = **2 GET**; `ies bloco N`
= **3 GET, zero POST**, quando N consta exatamente uma vez na página
consultada. Se N não estiver publicado nessa página, são **2 GET**
e recusa tipada `bloco_nao_encontrado_na_pagina`, sem concluir que N
não existe nas demais páginas.

`ies bloco listar --tipo interno --pagina 2` usa **2 GET + 1 POST**
único de navegação derivado dos controles de `frmBlocoLista`, exigindo
faixa contígua e mesmo total. Página 2 para assinatura/reunião ou
páginas acima de 2 são recusadas antes da rede. Uma página não
registrada no SQLite não é uma página observada vazia. Do mesmo modo,
detalhe não registrado não é detalhe com zero itens observados.

Os testes sintéticos de listagem, segunda página e detalhe já existem
em `tests/aies_bloco_cli.rs` e são executados no CI oficial, mas
**não foram reexecutados nesta revisão documental**. Confronto semântico
bilateral com corpos originais das cassettes: **NOT_RUN**. Treinamento
UnB/GO: **NOT_RUN**, sem novas requisições ao SEI.
