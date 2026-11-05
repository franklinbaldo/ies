---
type: ImplementationDecision
title: Lista de unidades no índice local
operation: unidade.listar
implementation: partial
status: proposed
scope: snapshots SQLite por instalação e unidade ativa
selected_strategy: web-lupa-com-snapshot-atomico
alternatives: web-lupa-com-snapshot-atomico,autocomplete-truncado,consulta-sempre-online
evidence_level: oraculo-todos-sem-replay-proprio
live_validation: not-run
---

# Lista de unidades para pesquisa offline (#244)

## Alternativas

O autocomplete foi rejeitado porque não enumera todas as unidades
(GO retorna no máximo 50). Consultar o SEI a cada pesquisa foi
rejeitado pelo custo e pela indisponibilidade offline. Escolhemos
ler a lupa completa uma vez e publicar um snapshot atômico no
índice local por `(instalacao, unidade_ativa)`.

## Identidade e integridade

A unidade ativa integra a chave porque o conjunto de destinos da
lupa pode variar com a unidade da sessão. A tabela `lista_unidade`
distingue ausência de snapshot de lista vazia legítima.
A tabela `unidade` guarda ID, sigla, descrição, órgão e época
Unix da leitura. Não guarda URL assinada nem sessão.

Um snapshot só substitui o anterior após o leitor verificar todas
as páginas e os totais. A transação apaga as linhas anteriores
e insere as novas de uma só vez. Falha do índice não deve derrubar
a leitura Web; a saída deverá avisar que não houve persistência.

## Acesso ao banco

A CLI deverá expor `ies indice caminho` em vez de ampliar
`ies status`: separa a descoberta de arquivo da inspeção de sessão.
Acesso SQL público somente por `SELECT`, sem promessa de suporte
para escrita. Alterar esquema exige nova versão e documentação.

## Limites e revisão

Esta primeira fatia cria o armazenamento e a pesquisa em memória
Rust, mas **não** liga o leitor HTTP, os comandos ou as sugestões
do envio. O contrato HTTP depende do replay da lupa no IES e dos
ensaios autorizados na UnB e em Goiás. Não inferir PASS live.
