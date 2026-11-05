---
type: CommandContract
title: Pesquisa de unidades no índice local
operation: unidade.pesquisar
command: ies unidade pesquisar TERMO
effect: read
implementation: partial
---

# Unidade pesquisar (#244)

`ies unidade pesquisar TERMO [--json]` filtra **somente** o snapshot
SQLite da instalação e unidade ativa da sessão local. O orçamento é
**zero requisições HTTP** e nenhuma escrita no SEI. Com dados já
guardados, funciona sem acesso à rede. O termo casa sem acentos e
sem diferença de caixa, por sigla ou descrição; siglas com `/`
continuam válidas, e a sigla exata vem primeiro.

Saída JSON: `origem: "indice"`, `lida_em` da lista integral original,
`unidade_ativa`, `total` de resultados filtrados,
`unidades` com `id`, `sigla`, `descricao` e `orgao`, e
`requisicoes: 0`. Zero resultados de snapshot existente é sucesso
(`total: 0`), nunca a mesma coisa que ausência de snapshot.

Um termo vazio é recusado; quando não existe lista, o código é
`lista_unidades_nao_guardada`, **não** há fallback Web nesta fatia.
Índice desligado/ilegível, origem não autorizada, unidade ativa sem
identidade e `--unidade` não verificável são recusados sem acessar
o SEI. Nenhum cookie nem URL assinada vai para a saída.

A captura integral da lupa e o preenchimento automático do índice
ainda são pendentes: esta é uma **fatia offline parcial**, não a
capacidade completa. [Decisão](../../docs/decisoes/unidade-pesquisar.md),
[evidência local](../evidencias/unidade-pesquisar.md).
