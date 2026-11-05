---
type: Evidence
title: Matriz de derivação da CLI de blocos
description: Cenários de teste e evidências para os leitores de blocos
---

# Matriz — bloco (#256)

| Operação IES | Caso oráculo Todos @323eb01 | Evidência | Resultado |
| --- | --- | --- | --- |
| `bloco listar` | `tests/paginas_reais/menu_blocos.html` e `blocos_lista.py` | `tblBlocos`, cabeçalhos por nome, legenda e linha | código/teste sintético; CI pendente |
| `bloco listar --tipo interno` | `docs/como-o-sei-funciona/15-telas-da-barra/blocos-internos.md` | ação `bloco_interno_listar` | código/teste sintético; live NOT_RUN |
| `bloco listar --tipo reuniao` | `blocos.py` / `bloco_reuniao_listar` | menu observado | código/teste sintético; live NOT_RUN |
| `bloco N` | `tests/paginas_reais/bloco_com_documentos.html` | `tblProtocolosBlocos` | código/teste sintético; replay NOT_RUN |
| Bloco interno | `tests/test_bloco_de_processos_010.py` | Processo sem Documento | teste sintético; live NOT_RUN |
| Falha fechada | `bloco-lista-obter-buscar-*` | número repetido, legenda truncada, URL assinada ambígua | teste sintético; replay NOT_RUN |

Orçamento do caso feliz inicial: lista = 2 GET, detalhe = 3 GET,
zero POST. Para `--pagina >1`, zero pedidos e erro tipado.
A contagem RED local é NOT_RUN (Rust ausente no sandbox);
a compilação e os gates dependem de CI do GitHub. Nenhuma
escrita ao vivo efetuada: UnB/GO NOT_RUN, 0 requisições reais.
