---
type: SEIObservation
title: Continuação e reconciliação da criação de processos em treinamento
description: Testes da Coruja e registro da confirmação de efeito incerto na UnB; sem repetir a escrita
operation: processo.criar
date: 2026-11-05
status: fail
---

# Continuação e reconciliação da criação de processos — issue #245

As seções anteriores ao ensaio estão em [relatório original](2026-11-05-processo-criar.md).
**Estado consolidado em 05/11/2026:** a confirmação havia retornado saída 3,
mas uma reconciliação independente, exclusivamente por leitura, **comprovou que
ela criou o processo 23106.001875/2026-93** (id 51731) na UnB de treinamento.
**Não repetir o POST criador.** As tabelas históricas abaixo conservam a
visibilidade que havia *na ocasião*; seus `DESCONHECIDO` e `NOT_RUN` não
representam o estado consolidado desta issue.

## Reconciliação independente comprovada — 🐘 Elefante (05/11/2026)

A conferência real foi documentada pelo proprietário na
[issue #245, comentário 6673196171](https://github.com/franklinbaldo/ies/issues/245#issuecomment-6673196171).
A evidência substitui, para o estado atual, a classificação histórica
`DESCONHECIDO` deste relatório; não substitui o fato de o POST original
ter retornado erro nem demonstra que a CLI já realize a reconciliação
automaticamente.

| Observação | Resultado confirmado |
| --- | --- |
| Origem | UnB **treinamento** |
| Reconciliação da escrita anterior | **PASS — criação aplicada**, somente leitura |
| NUP residual | **23106.001875/2026-93** |
| ID do processo | **51731** |
| Tipo | `Teste: Nível de acesso só público` |
| Especificação | `Ensaio IES 245 - escrita unica em treinamento` |
| Nível / interessados | público / nenhum |
| Registro de geração | 05/11/2026, 18h48 (-03), usuário `aluno1`, `Unidade1`; andamento `Processo público gerado` |
| Duplicidade | **não observada** entre os Gerados conferidos; processo anterior `001874` (id 51727) de 11h19, outro tipo/especificação |
| Novo POST de escrita nesta reconciliação | **0** |
| Novo resíduo desta retomada | **0** (a reconciliação não cria processos) |

**Requisições efetivamente relatadas pelo 🐘:** `ies login` = **6**;
`ies unidade gerados` = **página 1, 66 processos**, contagem HTTP
dessa operação **null** (não informada no comentário); `ies processo`
para os dois NUPs de IDs maiores = **5 por processo** (10 consultas
HTTP). A quantidade total da reconciliação = **null**, pois falta a
contagem HTTP da operação Gerados. O total da tentativa original de
confirmação = **null**, não observado no erro. Não inferir valor para
nenhum desses totais. Todas as consultas da reconciliação foram de leitura;
abrir processo pode registrar recebimento/visualização.

A prévia histórica passou em 3 GET e a confirmação original ocorreu uma
única vez, encerrando com `efeito_incerto_na_criacao`/saída 3.
**O efeito está esclarecido pelo histórico da UnB: criação aplicada.**
O binário da leitura independente foi o da `main` `d93c9b6`.
Não houve retry de escrita. Não repetir o POST, mesmo após esta prova.

**Pendente para concluir a issue:** implementar e testar reconciliação
*automática na CLI* para falhas/respostas inesperadas após POST, com
comparação Controle/Gerados e desfechos aplicada, não aplicada e ambígua,
preservando causas e orçamento exato. O script manual de reconciliação
não satisfaz esse critério. Não há nova escrita autorizável pelo fluxo
histórico já executado. A outra instalação de treinamento continua
`NOT_RUN` nesta fatia.

**Workflow de investigação manual:** a reconciliação já está concluída.
O arquivo temporário `temporary-245-readonly-reconcile.yml` foi
retirado do branch na mesma fatia de atualização deste relatório. Não
houve nova execução desse workflow.

## Retomada 🦉 Coruja — proteção de hipóteses AJAX (05/11/2026)

| Prova / etapa | Status | Requisições reais ao SEI | Resíduo |
| --- | --- | ---: | --- |
| Testes sintéticos RED de ID/nome AJAX não publicáveis | NOT_RUN — `cargo`/`rustc` ausentes, DNS do sandbox indisponível | 0 | Nenhum |
| GREEN/compilação Rust local | NOT_RUN — ambiente sem compilador Rust | 0 | Nenhum |
| Gate canônico e replay no novo head da PR #283 | NOT_RUN — não apareceu workflow do novo SHA na consulta ao GitHub | 0 | Nenhum |
| Prévia autenticada UnB e outra instalação de treinamento | NOT_RUN — sem conectividade/sessão autenticada | 0 | Nenhum |
| Uma única confirmação `--confirmar` em treinamento | NOT_RUN — não houve prévia ao vivo | 0 | Nenhum |
| Releitura verificadora do NUP | NOT_RUN — nenhuma criação efetuada | 0 | Nenhum |

Os testes Rust foram **publicados antes da correção**, em commits
`c979189` (2 testes unitários) e `be2fb9c` (1 teste de transporte
com orçamento exato de 3 GET + 1 POST AJAX consultivo, zero POST
criador). O teste integrado exige saída tipada
`hipotese_legal_ajax_resposta_nao_reconhecida` quando o XML anuncia
uma URL como identificador. O código anterior selecionava tal valor
por unicidade ou por `ID#NOME`; a execução RED ainda não foi
observada — **não confundir dedução do código com RED executado**.

A implementação no head seguinte recusa ID/nome não publicável tanto
no caso de uma única opção quanto na escolha explícita. Há também
teste de recusa da escolha segura sobre uma lista contaminada. A
correção não introduz requisição nem escrita. Executar os gates e
o replay antes de integrar; os últimos CI verdes citados acima
pertencem a SHAs anteriores e não comprovam o estado atual.

**Total live nesta retomada: 0 requisições ao SEI, 0 POST criador,
0 processos criados/alterados e nenhum resíduo.** A autorização de
escrita permanece válida apenas em treinamento; a sequência
prévia → um único `--confirmar` → releitura ainda não aconteceu.

## Reconciliação obrigatória após confirmação incerta — UnB treinamento

**Evidência real de 05/11/2026 (UTC, 21h47–21h49):**
[workflow de ensaio 42115596616](https://github.com/franklinbaldo/ies/actions/runs/42115596616),
job `UnB treinamento - previa confirmacao unica e releitura`.
Essa observação é posterior às seções históricas NOT_RUN acima e as
**substitui** quanto ao estado live da issue. A origem conferida pelo
roteiro era exclusivamente `https://sei.treinamento.unb.br/sei`.

| Etapa | Resultado observado | Requisições do comando | Resíduo |
| --- | --- | ---: | --- |
| Login UnB treinamento | PASS, saída 0 | 6 | sessão privada transitória no runner |
| Prévia pública de `Teste: Nível de acesso só público` | PASS, saída 0, `confirmado: false` | 3 (3 GET, 0 POST criador) | nenhum processo criado na prévia |
| **Uma** chamada `--confirmar` | **FAIL / efeito incerto**, saída **3**, `efeito_incerto_na_criacao` | **não observado** na resposta de erro; POST criador possivelmente efetuado | **DESCONHECIDO** |
| Releitura independente do NUP | NOT_RUN: confirmação terminou com erro e não revelou NUP | 0 | **a verificar na caixa Gerados** |

O roteiro interrompeu a execução imediatamente após a confirmação
incerta, sem repetir a escrita nem tentar outro backend.
O valor de NUP **não foi observado**. Portanto, **não é correto
registrar resíduo zero nem alegar processo criado**. Antes de qualquer
nova confirmação deve-se reconciliar por **leitura somente** a
caixa Gerados da conta pública UnB e, quando possível, conferir
tipo/especificação `Ensaio IES 245 - escrita unica em treinamento`
contra o processo. Uma leitura sem correspondência não prova que
a escrita falhou.

**Mitigação:** removido o workflow temporário de disparo por push
no commit `44005cb2`; não há autorização para reexecutar a confirmação
incerta. Nova infraestrutura de teste offline/manual da mesma PR
possui marca durável antes do POST, impede reuso do mesmo registro,
faz a releitura independente comparando o JSON `numero` com o NUP
e recusa origens fora das instalações de treinamento. Não foi
usada para repetir a criação real.

**Contagem real conhecida:** 6 login + 3 prévia + 0 releitura = 9
requisições observadas; quantidade interna da confirmação incerta
**não observada**, portanto total HTTP global **desconhecido**.
Nenhuma produção foi acessada.

## Fatia de reconciliação somente leitura — Coruja (05/11/2026)

**Motivação:** a confirmação da UnB em [run 42115596616](https://github.com/franklinbaldo/ies/actions/runs/42115596616)
devolveu `efeito_incerto_na_criacao` (saída 3), sem NUP. A prévia anterior
mostrou outro texto de especificação que o POST de ensaio; não usar essa
prévia histórica como prova de igualdade dos argumentos da confirmação.
A escrita **não** será repetida.

**Código entregue:** `scripts/reconciliar_processo_criar.py` e 12 testes
sintéticos `scripts/tests/test_reconciliar_processo_criar.py`. O coletor
faz uma leitura de `unidade gerados` e até **12 consultas independentes**
`ies processo NUP --sem-documentos --sem-marcadores --sem-andamentos`,
com intervalo mínimo de 30 segundos; aceita só NUPs sem caracteres de
controle, IDs/números observados, lista da categoria e unidade conferidas.
Compara NUP, ID, unidade e especificação. Não executa `processo criar`,
`--confirmar`, retry, scripts JS nem requer REST/SOAP. Cada abertura de
processo pode marcar o processo como visualizado: a conciliação é uma
**leitura com efeito declarado**, não leitura sem efeito algum.

Um candidato encontrado é `POSSIVEL_RESIDUO`, não prova de causalidade
do POST anterior. Ausência ou varredura parcial é `INCONCLUSIVO`, nunca
`resíduo zero`. Falha na leitura interrompe a coleta sem repetição.
As contagens HTTP vêm somente das respostas, permanecendo `null`
quando não observadas. Nenhum conteúdo bruto de resposta ou subcódigo
que não corresponda à forma tipada é impresso.

**RED local:** 1 teste novo falhou antes da filtragem de subcódigo
inseguro na evidência da conciliação; a saída sintética continha uma
URL com marcador sensível e a versão inicial a publicava. **GREEN local:**
12/12 testes Python aprovados, `compileall` PASS. São testes
sintéticos, sem acesso ao SEI. Testes incluídos no job canônico
`replay-processo-criar` do CI. **Rust local: NOT_RUN**, sem
`cargo`/`rustc` e com DNS GitHub indisponível no sandbox;
**CI do novo head: a verificar**, sem inferir resultado de SHA anterior.

O workflow temporário `temporary-245-readonly-reconcile.yml`
passou a exigir `workflow_dispatch` explícito, sem gatilho `push` e
sem `--confirmar`; utiliza esse módulo e não imprime dados brutos
da CLI. **Não foi disparado nesta retomada** para não repetir
requisições sem conhecer o histórico de falhas da reconciliação anterior.
Deve ser retirado antes de integrar a PR conforme AGENTS.md.

| Etapa nesta fatia | Status | Requisições reais | Resíduo |
| --- | --- | ---: | --- |
| Testes Python de reconciliação | PASS (12/12 sintéticos) | 0 | nenhum adicional |
| Conciliação na UnB | NOT_RUN (sem execução autenticada nova) | 0 | **DESCONHECIDO**, herdado da confirmação incerta |
| Outro `--confirmar` | **BLOQUEADO**, não executar | 0 | **DESCONHECIDO** |
| Releitura independente live | NOT_RUN nesta fatia | 0 | **DESCONHECIDO** |

**Próximo passo:** antes de qualquer nova criação, conferir se a
reconciliação de leitura já foi executada e, se não houve falha nem
requisição repetida, executar **uma vez** a varredura de Gerados em
treinamento, registrar o resultado parcial e o orçamento observado.
Mesmo `INCONCLUSIVO` não autoriza repetir o POST de efeito incerto.

### Reforço de segurança posterior ao primeiro GREEN

Mais um teste novo demonstrou **RED local: 1 falha** quando uma
resposta sintética continha identificador de processo com URL assinável;
a conciliação ainda o apresentava como candidato. A correção limita
`id_procedimento` a 1–32 dígitos ASCII observados, sem eco da URL.
O primeiro RED de filtragem de subcódigos permanece documentado acima:
**2 falhas individuais observadas antes de suas respectivas correções**.
**GREEN final local: 13/13 testes Python PASS**, sem `PYTHONPATH` especial,
mais `compileall` PASS. Nenhuma requisição ao SEI e nenhuma escrita
nesta fatia; **o resíduo histórico continua DESCONHECIDO**.

## Continuação 🦉 — identidade da prévia e integração da main

O roteiro manual `scripts/ensaio_processo_criar.py` agora confere
**`especificacao` literal idêntica**, além de `tipo`, `nivel` e
`confirmado=false`, **antes** de reservar o registro e de permitir
um único `--confirmar`. Isso corrige a lacuna identificada na
revisão da PR #283: uma prévia histórica na UnB e sua confirmação
tinham especificações diferentes. Se `especificacao` não for
observada ou não coincidir, o roteiro recusa com `PREVIA_FAIL`,
sem POST criador. Uma contagem de requisições booleana também
não autoriza escrita. Contagens em relatórios e registro privado
só são publicadas se inteiros não negativos; outros tipos,
inclusive URLs arbitrárias, viram `null`.

**RED local antes do código:** 3 testes de identidade/contagem
inválida falharam; 3 testes de telemetria insegura falharam
numa segunda rodada. **GREEN local:** 6/6 testes desses cenários,
mais compilação Python. Os seis testes equivalentes foram
incorporados à suíte ativa nos commits `866a1e6` e `736a93e`
antes da correção `0b536e9`. Orçamento sintético no caminho
válido: prévia 3, confirmação 6 e releitura 2. Nas recusas de
prévia: só 3 requisições sintéticas e 0 POST criador.
Timeout após iniciar confirmação continua efeito incerto,
saída 3, sem retry.

**CI intermediário:** no merge `81f2429`, o gate Python
do ensaio passou **17/18** testes; a única falha foi no teste
antigo de timeout com especificação de fixture `Teste sintético`
e pedido `Teste`. A fixture foi alinhada no commit
`2a6a448` para continuar testando *timeout após prévia válida*
sem mudar a expectativa de saída 3 e sem ignorar teste.
Não extrapolar verde de outro SHA: exigir CI no head final.

**Integração:** merge da `main` no branch da PR Draft #283
em `81f2429`, preservando outras frentes e a infraestrutura
nova de `#[derive(Causa)]`; o erro novo de hipótese
legal recebeu `#[causa(nome = "hipotese_legal_informada_invalida")]`.
Sem rebase nem force-push. O workflow temporário de
reconciliação permanece manual, não executado, a retirar
antes da integração final.

| Etapa desta sessão | Status | Requisições reais novas | Resíduo |
| --- | --- | ---: | --- |
| Testes sintéticos RED/GREEN e compilação Python | PASS local | 0 | Nenhum novo |
| CI final Rust/OKF/replay | A verificar no SHA final | 0 | Nenhum novo |
| Prévia autenticada em treinamento | NOT_RUN | 0 | histórico inalterado |
| Novo `--confirmar` na UnB | BLOQUEADO: POST incerto prévio não pode ser repetido | 0 | **DESCONHECIDO** |
| Reconciliação somente por leitura da UnB | NOT_RUN | 0 | **DESCONHECIDO** |
| Releitura independente live | NOT_RUN | 0 | **DESCONHECIDO** |

**Total desta continuação: 0 requisições reais ao SEI,
0 escritas e 0 novas alterações.** O histórico continua
com login 6, prévia 3, confirmação com contagem desconhecida
e resíduo **DESCONHECIDO**. Não repetir o POST; antes de nova
escrita, reconciliar somente por leitura, sem repetir GET falhado
e sem inferir ausência de criação de varredura inconclusiva.
Nenhuma produção foi acessada. A issue permanece aberta.


## Retomada da Coruja — causa AJAX tipada e reconciliação da main

Esta fatia foi dedicada a erros tipados e regressão **offline**, sem
abrir SEI. O erro `ErroHipotese` passou a derivar `Causa`, a
propagar a causa concreta de transporte e a representar forma de
tela não reconhecida por `ErroHtml`. Foram acrescentados dois
testes sintéticos de folha/caminho e de HTTP 503. Os testes foram
publicados antes da implementação no commit `95bc21e`; o CI
foi interrompido por pushes posteriores e pela incompatibilidade
simultânea no replay do cadastro, portanto **RED isolado: NÃO AFERIDO**
(sem atribuir número de testes que não rodaram).
A importação da `main` foi feita por merge, preservando as
alterações concorrentes e a PR Draft #283.

| Etapa | Resultado nesta retomada | Requisições reais SEI | Resíduo |
| --- | --- | ---: | --- |
| Testes novos e correção tipada em Rust | Alterações enviadas; CI do head final a verificar | 0 | Nenhum novo |
| Replay do acervo | A verificar no CI do head final; sem contato com SEI | 0 | Nenhum novo |
| Prévia autenticada UnB / segunda instalação | NOT_RUN | 0 | Histórico inalterado |
| Reconciliação da caixa Gerados da UnB | NOT_RUN — verificar histórico antes de qualquer GET | 0 | **DESCONHECIDO** |
| Novo `--confirmar` na UnB | BLOQUEADO — confirmação real anterior teve saída 3 | 0 | **DESCONHECIDO** |
| Releitura independente de NUP | NOT_RUN | 0 | **DESCONHECIDO** |

**Total de requisições reais nesta retomada: 0**; **POSTs de
escrita: 0**; **nenhum resíduo novo criado pela retomada**.
O ensaio histórico na UnB permanece com login PASS (6 pedidos),
prévia PASS (3 GET), confirmação única com código 3 e quantidade
interna de pedidos não observada. Seu resíduo é **DESCONHECIDO**.
A reconciliação é exclusivamente por leitura, ainda não executada.
Não repetir POST de criação, não interpretar ausência de NUP como
ausência de efeito e não declarar PASS live a partir do CI.
O workflow temporário de reconciliação manual permanece em Draft
para investigação segura; deve ser removido antes de integração.

## Retomada 🦉 de 05/11 — PR #304 (após integração da #283)

Esta fatia implementou somente o **comparador offline** de retratos
completos de Gerados; **não** ativou reconciliação automática na CLI
nem executou ensaio autenticado.

| Etapa | Estado desta fatia | Requisições SEI | Escritas |
| --- | --- | ---: | ---: |
| Testes sintéticos Rust (8 cenários) | PASS no CI #42151869482 | 0 | 0 |
| Replay do acervo (CI) | PASS, sem conexão SEI | 0 | 0 |
| Prévia ao vivo em treinamento | NOT_RUN | 0 | 0 |
| Reconciliação automática CLI após POST | NOT_RUN — implementação pendente | 0 | 0 |
| Nova confirmação em treinamento | NOT_RUN — nenhuma escrita tentada | 0 | 0 |
| Verificação live de resíduo novo | NOT_RUN — nenhum novo resíduo | 0 | 0 |

**Total da sessão: 0 requisições ao SEI, 0 POST criador, 0 resíduo novo.**
A confirmação histórica da UnB (única; saída 3) **foi aplicada**,
com NUP `23106.001875/2026-93` e id `51731`, segundo a
[reconciliação exclusiva por leitura do 🐘](https://github.com/franklinbaldo/ies/issues/245#issuecomment-6673196171).
A confirmação antiga não pode ser repetida. Este relato atual
substitui as afirmações históricas de resíduo “DESCONHECIDO” no
fim das seções anteriores, sem apagar o estado que era conhecido
àquela altura. Requisições não aferidas historicamente seguem `null`.

A etapa de segurança anterior ao POST e seus testes RED estão no
[relatório de guarda pré-POST](2026-11-05-processo-criar-guarda-pre-post.md).

## Continuidade 🦉 em 06/11 — conferência dos candidatos

RED sintético: [CI 42173910486](https://github.com/franklinbaldo/ies/actions/runs/42173910486),
3 FAIL, 1636 PASS, 17 ignored preexistentes. Fatia posterior: leitura
por ID/NUP, árvore e propriedades com um GET por etapa, sem POST novo.
Live nesta sessão: **NOT_RUN** (0 requisições reais; 0 escritas;
0 resíduos novos). O resíduo histórico UnB 23106.001875/2026-93
permanece inalterado e não será recriado.
