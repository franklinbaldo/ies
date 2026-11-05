---
type: CommandContract
title: Pesquisa de unidades no índice local
operation: unidade.pesquisar
command: ies unidade pesquisar TERMO
effect: read
implementation: partial
---

# Unidade pesquisar (#244)

`ies unidade pesquisar TERMO [--json]` filtra o snapshot
SQLite da instalação e unidade ativa da sessão local. Com a lista
já guardada, o orçamento é **zero requisições HTTP** e nenhuma escrita
no SEI; funciona sem acesso à rede. Sem snapshot, faz uma única
coleta integral via lupa Web antes de pesquisar, com o mesmo orçamento
e verificações de `unidade listar --atualizar`. O termo casa sem acentos e
sem diferença de caixa, por sigla ou descrição; siglas com `/`
continuam válidas, e a sigla exata vem primeiro.

Saída JSON: `origem: "indice"`, `lida_em` da lista integral original,
`unidade_ativa`, `total` de resultados filtrados,
`unidades` com `id`, `sigla`, `descricao` e `orgao`, e
`requisicoes: 0`. Zero resultados de snapshot existente é sucesso
(`total: 0`), nunca a mesma coisa que ausência de snapshot.

Um termo vazio é recusado; na ausência de lista, a leitura Web
é obrigatória e falha fechada quando sua integridade não se comprova.
Índice desligado/ilegível não impede a leitura Web, mas a saída marca
que ela não foi persistida. Origem não autorizada, identidade da unidade
ausente e `--unidade` não verificável permanecem recusas antes da rede.
Nenhum cookie nem URL assinada vai para a saída.

O preenchimento do índice usa substituição atômica da lista integral;
a validação ao vivo e o replay do acervo permanecem condicionantes
para declarar a capacidade completa. [Decisão](../../docs/decisoes/unidade-pesquisar.md),
[evidência local](../evidencias/unidade-pesquisar.md).
