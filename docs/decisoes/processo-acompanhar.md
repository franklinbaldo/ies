---
type: ImplementationDecision
title: Acompanhamento Especial pela barra e cadastro da interface Web
description: Inclusão por POST único, backend de retirada ainda sob teste e CLI bloqueada
operation: processo.acompanhar
implementation: partial
status: adopted-for-scope
scope: ies processo acompanhar NUP com inclusão publicada e backend de retirada em WIP; bloqueio CLI até validação
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
A tabela de gerência recusa ainda grupo duplicado entre IDs diferentes:
antes de concluir `ja_estava`, é indispensável uma identidade não ambígua.
Usa o erro já contratado `grupo_de_acompanhamento_ambiguo`, sem nova
requisição ou inferência de preferência pela primeira linha.
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

## Ausência comprovada do grupo em retirada (#254)

Na tabela completa observada, `--deixar --grupo G` seleciona **somente**
a linha com nome exato `G`. Se outros grupos estão presentes, mas `G`
não está, a escolha é uma retirada idempotente, sem derivar ação de
exclusão para outra linha. A prévia preserva `grupo=G`, publica
`ja_estava=true` e não inventa ID. A regressão sintética exige apenas
4 GETs e 0 POST tanto na prévia como no backend confirmado. Sem
`--grupo`, múltiplas linhas continuam ambíguas. A escrita pública
`--deixar --confirmar` segue fechada até validação segura em treinamento.

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

## Gate de habilitação da retirada

A implementação de backend desta quarta fatia é experimental: mesmo com
POST único, 302 validado e releitura independente, seus testes novos
não tiveram GREEN comprovado no head. Para não liberar escrita ao vivo
sem gate, a CLI mantém `--deixar --confirmar` recusado antes de abrir
sessão ou rede (`confirmacao_de_acompanhamento_ainda_bloqueada`). Os
testes injetados exercitam o backend diretamente, sem o SEI. A remoção
externa só será liberada quando CI, replay e prévia autenticada em
instalação de treinamento tiverem evidência concreta, não por inferência.

## Quinta fatia — script executável da ação de exclusão (🐢)

A presença textual de `function acaoExcluir` não demonstra que o navegador
publique uma função global para o `onclick`. Um `<script>` de dados
(`type=application/json`), um módulo (`type=module`) ou um script com
`src` não executa esse corpo inline como função global clássica. A
verificação da oferta passa a recusar tais casos, mantendo o `script`
clássico observado e a forma explícita `text/javascript`. Quatro cenários
sintéticos foram publicados antes da correção. RED e GREEN novos são
**NOT_RUN**: o sandbox está sem Rust e os commits do conector não
acionaram um run no head. A CLI continua a recusar `--deixar --confirmar`
antes da sessão e rede; nenhuma escrita é permitida por esta fatia.

## Sexta fatia — declaração global de exclusão, não texto inerte (🐢)

A revisão no head `943add8` identificou que a assinatura completa de
`acaoExcluir` dentro de `/* ... */` ainda poderia ser aceita como
função. A busca foi endurecida sem executar JS e sem regex: uma varredura
restrita encontra a assinatura somente fora de comentários e strings,
em posição de declaração global, recusando blocos aninhados, atribuição
de função e sintaxe de prefixo não reconhecida. A extração do corpo e
sua conferência estrita continuam independentes.

Cinco testes de fronteira foram publicados antes da correção
(`abd561a`), e um teste integrado adicional exige falha tipada com
**4 GETs exatos, 0 POST** para função inteiramente comentada.
Correção publicada em `ebb9858`. **RED isolado e GREEN deste head:
NOT_RUN**, pois o sandbox não possui Rust nem rede para instalá-lo e
nenhuma execução GitHub Actions foi encontrada. Não inferir sucesso
do CI antigo. A CLI de retirada confirmada permanece bloqueada antes
da sessão e rede; replay e live ainda faltam.

## Sétima fatia — redirect da inclusão não é prova isolada (🐢)

O POST de inclusão já exigia releitura com exatamente uma linha nova,
mas **ignorava o destino do redirect**: um 302 ao cadastro ou para outro
`id_procedimento`, seguido de lista com linha coincidente, podia ser
classificado como sucesso. Dois testes de transporte sintético,
publicados antes da implementação em `4bb0843`, reproduziram o
problema: [RED 42163834327](https://github.com/franklinbaldo/ies/actions/runs/42163834327)
com **2 testes falhando, 1.654 passando e 17 ignorados preexistentes**.

O fluxo agora reutiliza a conferência do 302 ao gerenciador de
acompanhamento do processo correto, já usada na retirada. A releitura
independente permanece obrigatória. Uma falha de transporte depois
do envio continua passível de conciliação somente por leitura; uma
resposta explícita de redirect para outro processo ou operação
termina com efeito incerto, nunca com sucesso. Não executa JavaScript,
não usa regex ou retry e não acrescenta requisição.

Replay específico das cassettes originais e ensaio ao vivo estão
pendentes. `--deixar --confirmar` continua fechado na CLI antes
da rede até homologação segura em treinamento.


## Nona fatia — resposta de recusa não vira aceite de inclusão (06/11/2026, 🐢)

**Problema.** A conta de treinamento é compartilhada. Entre um POST e
o GET verificador, outro agente pode incluir uma linha com o mesmo grupo
e observação. A condição anterior `resposta.is_err()` aceitava como
candidata à reconciliação **qualquer** falha do envio, até uma recusa
explícita do SEI por `txaInfraValidacao` (HTTP 200) ou HTTP 403. Uma
linha coincidente lida depois podia ser atribuída indevidamente ao
POST recusado.

**Decisão.** A reconciliação positiva após resposta de erro só é
admitida para `ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)`,
quando não houve resposta do servidor e o efeito é desconhecido. Resposta
explícita de recusa não é aceite; mesmo com linha coincidente, a operação
falha fechada em `EfeitoIncerto` (saída 3). O aceite normal continua a
exigir um 302 ao gerenciador do mesmo processo e a **releitura exata**.
Não há POST adicional, retry ou JavaScript executado.

**RED observado antes da correção:** [CI 42175520247, tentativa 2](https://github.com/franklinbaldo/ies/actions/runs/42175520247/attempts/2),
head somente com testes `d3c4076`: **2 testes novos falharam,
1.668 passaram, 18 ignorados preexistentes**; Rustfmt, Clippy e OKF
passaram. Os testes usam respostas sintéticas com `txaInfraValidacao`
e HTTP 403, exigem o mesmo orçamento de **5 GETs + 1 POST + 1 GET
verificador** e demonstram que uma recusa não pode virar sucesso só
por uma linha coincidente. A alteração de produção veio depois
(`4f2064c`). A comprovação GREEN pertence ao CI do head final.

**Live:** esta fatia só rodou em transporte injetado. Prévia autenticada,
confirmação e releitura no SEI de treinamento: **NOT_RUN**, 0 requisições
reais e nenhum resíduo. A retirada `--deixar --confirmar` permanece
fechada na CLI antes de sessão/rede.


## Decima segunda fatia — grupo ja acompanhado dispensa novo cadastro (06/11/2026, 🐢)

**Observacao:** a estrutura da lista do gerenciador na UnB, descrita em
`todos/tests/test_processo_acompanhar_010.py` @323eb01, identifica cada
acompanhamento por grupo, enquanto o cadastro serve para preparar uma
nova inclusao. A issue exige resultado idempotente quando o grupo ja
esta acompanhado na unidade. A implementacao anterior abria o cadastro
antes de verificar esse fato; uma lista valida sem botao Adicionar
produzia erro apesar de o grupo ja estar observado.

**Escolha:** para `--grupo` explicito e igual a uma linha validada da
lista, devolver estado observado sem abrir cadastro e sem POST. Sem
`--grupo` explicito, a selecao do item em branco ainda precisa da tela
do cadastro; outros grupos exigem oferta exata e formulario derivado.
A verificacao usa somente o HTML recebido, sem executar JavaScript.

**RED comprovado:** [CI 42204125145](https://github.com/franklinbaldo/ies/actions/runs/42204125145),
commit somente de teste `d3a4e96`: 1 falha nova, 1.721 testes
aprovados, 23 ignorados ja existentes. Rustfmt tambem apontou somente
uma linha vazia excedente, corrigida separadamente antes do GREEN.
Orcamento novo: **4 GETs / 0 POSTs**, sem acesso real ao SEI.

**Diagnóstico da saída 3:** manter o subcódigo e publicar apenas contagem e motivo tipados; sem URL, cookie ou retry.

## Integridade da lista e dos controles (#254)

A validação posicional da tabela, a idempotência e as guardas de oferta
inerte na inclusão e retirada estão detalhadas no
[guia de integridade das ofertas](processo-acompanhar-integridade-ofertas.md).
