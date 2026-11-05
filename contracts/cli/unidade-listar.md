---
type: CommandContract
title: Lista local das unidades do SEI
operation: unidade.listar
command: ies unidade listar [--atualizar]
effect: read
implementation: partial
---

# Unidade listar (#244)

**Fatia implementada:** sem `--atualizar`, lê o snapshot integral já
guardado no índice SQLite (esquema 3), sem enviar requisições ao SEI.
A identidade é a origem HTTPS autorizada e o ID da unidade ativa
da URL privada do Controle, ambos lidos localmente e sem publicar
a URL ou o cookie. A lista só pertence à unidade e instalação exatas.

`--json` retorna `origem: "indice"`, `lida_em` (época Unix),
`total`, `unidade_ativa`, `unidades` (cada uma com `id`, `sigla`,
`descricao`, `orgao`) e `requisicoes: 0`.
Sem JSON, mostra a contagem e as linhas com esses quatro campos.

**Lacunas:** `--atualizar` falha fechado com
`leitura_integral_da_lupa_pendente`, sem usar uma lista antiga como
se estivesse atualizada. Sem snapshot, falha com
`lista_unidades_nao_guardada`; a obtenção automática na lupa ainda
não está ligada. Índice desligado/ilegível, origem inválida ou
identidade da unidade ausente também falham tipadamente.
`--unidade` explícita é recusada com
`unidade_pedida_nao_verificavel_offline`: a CLI não finge ter
verificado no SEI a sigla exibida. Nenhum caso faz retry.

A integridade, a consulta SQL pública somente leitura e a
migração estão em [tabela de unidades](../../docs/arquitetura/0003-indice-local-unidades.md).
Não implica que o snapshot ainda seja o mais recente.
[Decisão](../../docs/decisoes/unidade-listar.md) e
[evidência local](../evidencias/unidade-listar.md).
