---
type: CommandContract
title: Leitura e escrita de comentários em processos
description: Contrato da operação processo comentarios e processo comentar com leitura da tabela e reconciliação da escrita
---

# Comentários do processo (#257)

Comandos pretendidos: `ies processo comentarios NUP [--json]` e
`ies processo comentar NUP --texto T [--confirmar]`. A CLI Web ainda **não
está conectada** nesta fatia; os leitores e a conferência da escrita
são código Rust compilado e exercitado em testes sintéticos.

## Observação

Oráculo: Todos `src/todos/comentarios.py` e
`tests/test_comentarios_010.py` @323eb01, observação de UFRJ 4.1,
GO 5.0 e [acervo UnB](../../cassettes/unb/comentarios-ciclo-unb--c7d0c8a53b5e.md).

O link da barra da árvore usa `comentario_listar`. Sem comentários,
a lista redireciona ao formulário `frmComentarioCadastro`, contendo
`txaDescricao` e botão `sbmCadastrarComentario`. Com comentários,
a tela contém `form#frmComentarioLista` e `table#tblComentarios`,
cujo caption declara o total. A visão `hdnVisualizacao` vale P
(processo) ou T (processo e documentos). No 4.x a linha de dados
vem seguida de `tr.trComentario`; no 5.0 há coluna Texto na
própria linha.

## Leitura

A leitura deverá abrir o processo por um caminho autorizado já observado,
seguir exclusivamente o link publicado, conferir a identidade do processo,
e projetar id, data/hora, unidade, usuário e texto; protocolo e tipo
apenas se publicados. O leitor offline:
- recusa lista sem tabela, cabeçalho, legenda ou visão identificável;
- recusa ids duplicados, texto sem linha correspondente, divergências
  entre total da legenda e número de linhas (inclusive paginação);
- distingue formulário de cadastro vazio de lista;
- não transforma `null` ou tabela ausente em lista vazia.

Orçamento esperado da leitura Web: abertura do processo
(1 a 4 requisições segundo o caminho do índice) + um GET
`comentario_listar` quando há lista. Desvios e eventual 302 ao cadastro
exigem regra por operação antes da integração.

## Escrita

A prévia lê a tela de cadastro e o formulário sem enviar a gravação.
`--confirmar` só pode fazer um único POST observado,
sem retry. A resposta aceita é 302 para `comentario_listar` com
`id_comentario`; isso sozinho **não comprova efeito**.
A reconciliação exige nova leitura completa com uma única ocorrência
do id aceito **e texto igual byte a byte ao pedido**.
Resposta 5xx, falha de transporte após POST, ausência de id ou impossibilidade
de releitura resultam em **efeito incerto (saída 3)**, sem repetir escrita.
O fluxo Web ainda não está implementado: **não há POST habilitado**.

## Subcódigos desta fatia

| Subcódigo | Fato |
| --- | --- |
| `html_dos_comentarios_nao_interpretado` | HTML ou atributo inválido |
| `tela_de_comentarios_nao_reconhecida` | nem lista nem cadastro publicado |
| `formularios_de_comentarios_repetidos` | formulário repetido |
| `visao_dos_comentarios_nao_publicada` | P/T ausente ou inválida |
| `tabela_de_comentarios_ausente` | tabela não publicada |
| `tabelas_de_comentarios_repetidas` | tabela duplicada |
| `legenda_dos_comentarios_invalida` | total não legível |
| `cabecalho_dos_comentarios_invalido` | colunas essenciais não identificadas |
| `linha_do_comentario_invalida` | linha fora da estrutura |
| `texto_do_comentario_nao_publicado` | falta a linha de texto 4.x |
| `id_do_comentario_repetido` | ID repetido |
| `lista_de_comentarios_incompleta` | total diferente do número de linhas |
| `id_do_comentario_ausente_no_aceite` | id do 302 ausente |
| `comentario_nao_visivel_na_releitura` | id aceito não visto |
| `comentario_novo_repetido_na_releitura` | mais de uma ocorrência |
| `texto_do_comentario_diferente_do_pedido` | texto não corresponde |

## Estado e gates

**Fatia parcial offline**. Os módulos `src/processo_comentarios.rs` e
`src/processo_comentar.rs` são componentes funcionais, mas
nenhum comando do usuário é declarado implementado até a conexão com
árvore, transporte, formulários e testes de orçamento. Replay e live
seguem NOT_RUN. Não validar integração apenas pela compilação.
