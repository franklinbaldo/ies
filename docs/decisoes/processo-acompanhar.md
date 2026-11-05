---
type: ImplementationDecision
title: Acompanhamento Especial pela barra e cadastro da interface Web
description: Estratégia parcial para a prévia da inclusão e retirada com seleção tipada
operation: processo.acompanhar
implementation: partial
status: adopted-for-scope
scope: ies processo acompanhar NUP sem confirmar, com seleção do grupo oferecido, conferência da lista e prévia de inclusão ou retirada; a escrita permanece bloqueada até POST único e leitura de verificação
selected_strategy: web-barra-cadastro-tabela
alternatives: web-barra-cadastro-tabela,rest-acompanhamento,simular-estado
evidence_level: todos-unb-e-indices-de-cassettes
live_validation: not-run
---

# Acompanhamento Especial (Refs #254)

## Contexto

O Todos `src/todos/acompanhamento.py` e o teste
`tests/test_processo_acompanhar_010.py` @323eb01 registram a UnB
em 15–16/10/2026: a barra oferece `acompanhamento_gerenciar`,
que redireciona ao cadastro na ausência de acompanhamento, ou traz
a tabela `Tabela de Acompanhamentos.`. O `btnAdicionar` abre
`frmAcompanhamentoCadastro` com `selGrupoAcompanhamento`,
`txaObservacao` e `hdnIdProtocolo`. Os grupos dependem da unidade.

## Alternativas consideradas

1. **Web barra/cadastro/tabela (escolhida):** usa somente URLs assinadas
   servidas, valida origem, ação e processo, trata 302 como dado,
   não executa JS e recusa HTML truncado.
2. **REST:** precisaria de contrato e medição de backend habilitado;
   não foi escolhido para esta interface.
3. **Simular o estado:** inferir vazios da ausência de tabela poderia
   afirmar ausência em HTML cortado; rejeitado.

## Comparação

| Critério | Web | REST | Simulação |
| --- | --- | --- | --- |
| Caminho medido na UnB | sim | não | não |
| Opções por unidade observadas | sim | a verificar | não |
| GET adicional ao cadastro | até 1 | indefinido | 0 |
| Risco de inventar ausência | baixo, falha fechado | a verificar | alto |

## Decisão

A abertura reutiliza `processo_obter`; a barra do nó raiz só fornece
uma URL se tiver exatamente um link válido, conferido pelo leitor
`arvore_processo::acompanhar`. O gerenciador fornece a tabela ou o
302 para cadastro. A prévia valida a identidade do formulário, escolhe
nome exato ou branco observado, e exige identidade única na remoção.
A contagem inclui todos os GETs. **Nenhum POST é enviado nesta fatia**;
`--confirmar` falha antes de ler a sessão privada.

A reconciliação pura, com testes de identidade e rejeição de alterações
não solicitadas, está preparada para a próxima fatia. Não é
evidência de reconciliação da escrita ao vivo.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-acompanhar.md).
- [Matriz de evidências](../../contracts/evidencias/processo-acompanhar.md).
- [Testes de interface](../../tests/cli_processo_acompanhar.rs).
- [Testes de parsing e reconciliação](../../src/processo_acompanhar_tests.rs).
- [Índice da gravação da UnB](../../cassettes/unb/processo-acompanhar-unb--4e19a6146bc4.md):
  45 interações, das quais 39 GET e 6 POST; o índice não prova os corpos.
- [Relatório live](../validacao-live/2026-11-05-processo-acompanhar.md).

## Limitações e revisão

RED local NOT_RUN (sem toolchain e DNS no sandbox); CI só comprova
o head ao qual estiver vinculado. Replay das cassettes e validação
live na UnB e numa segunda instalação: NOT_RUN, 0 requisições ao SEI.
Nenhuma escrita e nenhum resíduo foram produzidos.

Antes de liberar `--confirmar`, implementar o POST único derivado
do formulário servido, validação da resposta (inclusive 302 e
recusas da Infra), uma nova leitura do mesmo processo/unidade,
conciliação tipada com saída 3 para efeito incerto, testes de orçamento
exato e ensaio exclusivamente em treinamento.
