---
type: CommandContract
operation: processo.comentarios
implementation: partial
title: Leitura e escrita de comentários em processos
description: Contrato da operação processo comentarios e processo comentar com leitura da tabela e reconciliação da escrita
---

# Comentários do processo (#257)

`ies processo comentarios NUP --json` está conectado à leitura Web:
abre a árvore pela identidade do processo, exige o link `comentario_listar`
publicado na barra, faz um GET e só declara `lista_completa=true` quando a
tabela e o total foram integralmente conferidos. A ausência de link não vira
lista vazia. `ies processo comentar NUP --texto T` tem **prévia de leitura**
para lista já publicada com comentários: abre o link `Novo` servido pela lista,
confere o formulário `frmComentarioCadastro` e os controles de descrição e
cadastro sem realizar POST. `--confirmar` envia um único POST em instalações públicas de treinamento,
e exige uma releitura completa com o id aceito e o texto exato.

## Observação

Oráculo: Todos `src/todos/comentarios.py` e
`tests/test_comentarios_010.py` @323eb01, observação de UFRJ 4.1,
GO 5.0 e [acervo UnB](../../cassettes/unb/comentarios-ciclo-unb--c7d0c8a53b5e.md).

O link da barra da árvore usa `comentario_listar`, extraído sem montar URL e validado quanto à origem, identidade do processo e unicidade. Sem comentários,
a lista redireciona ao formulário `frmComentarioCadastro`, contendo
`txaDescricao` e botão `sbmCadastrarComentario`. Com comentários,
a tela contém `form#frmComentarioLista` e `table#tblComentarios`,
cujo caption declara o total. A visão `hdnVisualizacao` vale P
(processo) ou T (processo e documentos). No 4.x a linha de dados
vem seguida de `tr.trComentario`; no 5.0 há coluna Texto na
própria linha.

## Leitura

A leitura abre o processo por um caminho autorizado já observado,
segue exclusivamente o link publicado na árvore conferida, sem reconstruir
assinaturas, e projeta id, data/hora, unidade, usuário e texto; protocolo
e tipo apenas quando publicados. O leitor estrutural:
- recusa lista sem tabela, cabeçalho, legenda ou visão identificável;
- recusa ids duplicados, texto sem linha correspondente, divergências
  entre total da legenda e número de linhas (inclusive paginação);
- distingue formulário de cadastro vazio de lista;
- não transforma `null` ou tabela ausente em lista vazia.

Orçamento de leitura: abertura do processo pelo fluxo comum (a depender
do índice) e **um GET** `comentario_listar` quando a barra publica o link.
Lista truncada, ausência de ação, resposta 302 para cadastro e telas não
reconhecidas são recusadas com erro tipado, **sem afirmar lista vazia**.
A interpretação positiva do redirecionamento ao cadastro continua pendente.

## Escrita

A prévia implementada exige lista reconhecida, link único assinado de
`comentario_cadastrar` com a mesma identidade do processo e formulário de
cadastro publicado. O redirecionamento 302 para o cadastro quando a lista
está vazia ainda não é aceito pela prévia e não é interpretado como ausência
comprovada de comentários. O fluxo de confirmação é habilitado somente em treinamento, após a conferência da tela.
O modo `--confirmar` faz no máximo um POST observado, sem retry. A resposta aceita é 302 para `comentario_listar` com
`id_comentario`; isso sozinho **não comprova efeito**.
A reconciliação exige nova leitura completa com uma única ocorrência
do id aceito **e texto igual byte a byte ao pedido**.
Resposta 5xx, falha de transporte após POST, ausência de id ou impossibilidade
de releitura resultam em **escrita não confirmada (saída 1)**, sem repetir escrita.
**O POST está disponível exclusivamente em treinamento**; o CI exercita a
hipótese sintética em HTTPS local, mas não a validação live.

## Subcódigos desta fatia

| Variante ou `onde` (stderr) | Fato |
| --- | --- |
| `barra_dos_comentarios_fora_da_forma_observada` | barra do processo fora da forma observada |
| `barras_dos_comentarios_repetidas_na_arvore` | mais de uma barra no nó PROCESSO |
| `links_dos_comentarios_repetidos_na_barra` | links de comentários repetidos |
| `endereco_dos_comentarios_nao_reconhecido` | link fora do formato assinado observado |
| `ErroLinkDosComentarios::EnderecoDeOutroProcesso` | identidade do processo no link divergente |
| `html_dos_comentarios_nao_interpretado` | HTML ou atributo inválido |
| `tela_de_comentarios_nao_reconhecida` | nem lista nem cadastro publicado |
| `formularios_de_comentarios_repetidos` | formulário repetido |
| `ErroComentarios::VisaoNaoPublicada` | P/T ausente ou inválida |
| `tabela_de_comentarios_ausente` | tabela não publicada |
| `tabelas_de_comentarios_repetidas` | tabela duplicada |
| `legenda_dos_comentarios_invalida` | total não legível |
| `cabecalho_dos_comentarios_invalido` | colunas essenciais não identificadas |
| `linha_do_comentario_invalida` | linha fora da estrutura |
| `TextoDoComentarioNaoPublicado` | falta a linha de texto 4.x |
| `id_do_comentario_repetido` | ID repetido |
| `ErroComentarios::ListaIncompleta` | total diferente do número de linhas |
| `FalhaDaConferencia::IdDoAceiteAusente` | id do 302 ausente |
| `ComentarioNaoVisivelNaReleitura` | id aceito não visto |
| `FalhaDaConferencia::ComentarioNovoRepetido` | mais de uma ocorrência |
| `FalhaDaConferencia::TextoDiferenteDoPedido` | texto não corresponde |

## Estado e gates

**Leitura Web (BDD-257-01) e prévia/confirmação local (BDD-257-02 sintético)**. O leitor offline
`src/processo_comentarios.rs` foi conectado em
`src/processo_comentarios_web.rs` / `src/processo_comentarios_cli.rs`,
com SQLite evolutivo v33 e projeção do anti-IES. Os testes de integração
`tests/aies_comentarios_cli.rs` exercitam CLI real ↔ HTTPS local ↔
anti-IES ↔ SQLite, incluindo recusa de foto desconhecida e página parcial.
São hipóteses **sintéticas** e não demonstram paridade independente com
respostas de cassette. A inspeção estrutural das cassettes UnB/GO soma 76
interações, SHA-1/SHA-256 conferidos; confronto semântico independente e
SEI live: **NOT_RUN**. O caso de lista vazia/redirecionamento permanece pendente. O BDD-257-03
possui regressão sintética de 503 durante a releitura após o POST,
com saída 1 e nenhuma repetição, e depende de gate verde no head final.
Não foi realizado teste live.
