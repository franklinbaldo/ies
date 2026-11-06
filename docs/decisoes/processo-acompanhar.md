---
type: ImplementationDecision
title: Acompanhamento Especial pela barra e cadastro da interface Web
description: Estratégia Web de prévia, POST único e releitura da inclusão e retirada
operation: processo.acompanhar
implementation: partial
status: adopted-for-scope
scope: ies processo acompanhar NUP com inclusão e retirada por um POST cada e uma releitura; live pendente
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
A contagem inclui todos os GETs. Para incluir, a prévia também
confere se o primitivo `formulario_web` pode derivar o POST exato da
tela; somente `--confirmar` consome a submissão uma vez. O formulário
é escolhido pelo identificador `frmAcompanhamentoCadastro` e a
submissão reenvia ocultos, grupo, observação e botão conforme servidos.
Nenhum 302 é seguido e nenhuma falha de rede dispara retry.
O gerente é relido uma vez após o POST; só uma linha nova com
grupo/observação correspondentes e linhas anteriores preservadas
comprova sucesso. Sem prova, a saída é 3. A retirada confirmada deriva um POST da mesma tela da lista, com a ação
alterada por `acaoExcluir` apenas inspecionado, sem executar JavaScript. A nova prévia de retirada não se
baseia somente numa checkbox: exige a oferta `acaoExcluir` na linha,
o formulário `frmGerenciarAcompanhamento`, o oculto reescrito pelo
script `hdnInfraItemId` (nome e ID únicos no DOM, não desabilitado)
e a ação assinada `acompanhamento_excluir`
para o mesmo processo. Esses elementos são examinados como texto e DOM,
sem executar JavaScript ou publicar hash. Falha fechada nas variantes
não observadas. A resposta 302 de exclusão tem de apontar ao mesmo
processo; a leitura nova deve comprovar a remoção exata, preservando
todas as outras linhas. A retirada final pode retornar 302 ao cadastro:
a operação faz GET explícito e valida o formulário do processo antes
de reconhecer a lista vazia. Sem as duas evidências, a saída é 3.
O fluxo está implementado em branch, mas **sem GREEN ou live comprovado**
para esta nova fatia.

A suíte injetada mede 5 GETs para prévia e 3 GETs de abertura +
GET de gerenciar + GET de cadastro + POST + GET de releitura (7
requisições) para inclusão. Uma falha na resposta do POST não elimina
a necessidade de verificação. Testes sintéticos não substituem a
validação nas instalações de treinamento.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-acompanhar.md).
- [Matriz de evidências](../../contracts/evidencias/processo-acompanhar.md).
- [Testes de interface](../../tests/cli_processo_acompanhar.rs).
- [Testes de parsing e reconciliação](../../src/processo_acompanhar_tests.rs).
- [Testes de fluxo e orçamento](../../src/processo_acompanhar/tests_fluxo.rs).
- [Índice da gravação da UnB](../../cassettes/unb/processo-acompanhar-unb--4e19a6146bc4.md):
  45 interações, das quais 39 GET e 6 POST; o índice não prova os corpos.
- [Relatório live](../validacao-live/2026-11-05-processo-acompanhar.md).

A terceira fatia restringe a inspeção de `acaoExcluir` ao corpo da própria
função servida: assinatura observada, chaves balanceadas sem interpretar
delimitadores dentro de strings, um único ramo `if(confirm(...))`,
atribuição do identificador ao oculto, alteração da ação assinada e
`submit()` nessa ordem, sem instruções extras. A busca anterior de
substrings no `script` inteiro aceitaria três instruções numa segunda
função ou num ramo morto. Os testes sintéticos reproduzem ambas as
armadilhas e chaves no texto de confirmação. A lógica é propositalmente
estrita para a forma observada na UnB; outras formas falham fechadas,
sem executar JavaScript. O reforço da prévia foi preservado no percurso confirmado, com
POST único e verificação independente. A habilitação requer revisão
antes de uso real, pois CI/live desta fatia ainda não foram observados.

## Limitações e revisão

RED desta segunda fatia: 1 teste novo falhou no CI 42118873078 antes da implementação; compilação Rust local NOT_RUN (sem toolchain e DNS no sandbox); CI só comprova
o head ao qual estiver vinculado. Replay das cassettes e validação
live na UnB e numa segunda instalação: NOT_RUN, 0 requisições ao SEI.
Nenhuma escrita e nenhum resíduo foram produzidos.

Ainda faltam replay integral do acervo e teste ao vivo em
treinamento, com prévia anterior ao `--confirmar`, envio único,
releitura e registro de resíduo. A retirada agora deriva `acompanhamento_excluir` da linha e confere sua
identidade, sem copiar a arquitetura do Todos. Falta comprovar RED/GREEN,
replay do acervo e validação em treinamento. A PR permanece DRAFT.

## Quarta fatia — remoção verificada (🐢)

A implementação expõe a `action` assinada a partir da inspeção estrita da
função `acaoExcluir` sem JS, valida também a `action` original do
formulário no primitivo `formulario_web`, deriva apenas o corpo
`hdnInfraItemId=<id>` e consome uma única submissão. Evita
seguir redirects e não repete POST após falha de rede. A verificação
é uma nova leitura, não a confiança no status HTTP.

Seis testes de transporte sintético foram publicados **antes** do código:
POST único com releitura da tabela, retirada do último com GET de cadastro,
302 para outro processo, falha no POST, linha persistente e cadastro
incompleto. Seus orçamentos esperados são 6 ou 7 requisições, conforme
o redirect. A contagem RED nova e o GREEN são **NOT_RUN / não observados**,
pois o CI canônico não foi disparado pelos commits desta fatia e o
sandbox carece de Rust e rede. Não inferir gates verdes. Também não
houve ensaio real: 0 requests e 0 resíduos no SEI.
