---
type: SEIObservation
title: Estado da validação de iniciar processo
operation: processo.criar
date: 2026-11-05
status: not-run
---

# Iniciar processo — validação da issue #245

| Etapa | Resultado | Requisições IES |
| --- | --- | ---: |
| Inspeção offline do Todos e captura SEI 4.0.9 | OBSERVADO INDIRETAMENTE | 0 |
| RED original da PR #249 (falhas antes do código) | NOT_RUN — contagem histórica não aferida | 0 |
| RED de regressão da PR #283 | PASS como evidência: CI 42094129597 registrou 2 falhas novas antes da correção | 0 |
| Teste de confirmação 4.1 (2 POST de naturezas distintas) | PASS sintético no CI 42094961462 (7 requisições simuladas) | 0 |
| Rejeição de Location após escrita com ação/ID repetidos | PASS sintético no CI 42094961462 (4 requisições simuladas, um POST criador) | 0 |
| Testes Rust com transporte sintético (4.0/4.1) | PASS no CI #948 | 0 |
| Compilação, Clippy, suíte Rust e subcódigos | PASS no CI #948 | 0 |
| Gate completo no head 1d263ee | PASS (run 42069555057) | 0 |
| Recusa HTTP do GET preserva status (integração) | PASS no teste sintético | 0 |
| Replay integral das cassettes publicadas | PASS offline em 42102035089 (15 cassettes, 83 respostas; não prova escrita) | 0 |
| Prévia ao vivo em UnB/UFRJ | NOT_RUN | 0 |
| Criação ao vivo | NOT_RUN — autorizada em treinamento (05/11/2026), ainda não executada | 0 |
| Leitura posterior de NUP criado | NOT_RUN | 0 |

Nenhuma requisição foi enviada a qualquer instalação do SEI nesta fatia.
O sandbox sem Rust e com DNS indisponível não executou testes locais;
o CI compilou e executou a suíte. A tentativa de criar workflow
para ensaio em treinamento foi bloqueada pelo controle de segurança
da ferramenta antes de qualquer execução; a automação não a contornou.
A escrita ao vivo em treinamento está autorizada pelo proprietário
(05/11/2026): prévia, um único `--confirmar` e leitura verificadora,
com o resíduo registrado aqui. Ainda não foi executada.

O código da CLI, da prévia e da confirmação foi implementado
na PR #249 e testado em transporte injetado (sem rede real).
Os testes offline NÃO certificam paridade live IES/Todos. As linhas
históricas abaixo distinguem o estado de cada fase do estado atual.
RED antes da implementação: contagem não medida. A revisão
final do CI no head atualizado precisa ser consultada no GitHub.

## Nova fatia #283 (05/11/2026)

A revisão de URL assinada agora recusa parâmetros duplicados,
mesmo quando um dos valores seja válido. Na rejeição anterior à
consulta do cadastro, o orçamento testado é **2 GET, 0 POST**.
Na rejeição posterior ao POST criador, o teste demonstra **4 pedidos,
com um POST criador**: o fluxo não segue a Location ambígua e retorna
`efeito_incerto` com código 3. A confirmação 4.1 válida tem
**7 pedidos** (5 GET, 1 POST de seleção e 1 POST criador).
Os números são do transporte sintético, **não** do SEI vivo.

O RED próprio da fatia foi registrado no CI
[42094129597](https://github.com/franklinbaldo/ies/actions/runs/42094129597):
**2 falhas** (1392 testes passaram, 13 ignorados já existentes).
A compilação local e o RED local permaneceram **NOT_RUN**.


## Fase AJAX — hipótese legal restrita (05/11/2026)

| Etapa | Resultado | Pedidos sintéticos | Requisições ao SEI |
| --- | --- | ---: | ---: |
| Observação da declaração AJAX da captura Todos @323eb01 | PASS — fonte indireta | 0 | 0 |
| RED: 4 cenários de AJAX antes da implementação | PASS como evidência — [CI 42097640389](https://github.com/franklinbaldo/ies/actions/runs/42097640389), 4 falhas novas | 0 | 0 |
| RED e compilação locais | NOT_RUN — Rust indisponível e DNS sem resolução | 0 | 0 |
| Prévia restrita com uma hipótese no AJAX | PASS sintético no [CI 42098895457](https://github.com/franklinbaldo/ies/actions/runs/42098895457) | 4 (3 GET + 1 POST de consulta) | 0 |
| Confirmação restrita com AJAX + releituras | PASS sintético no mesmo CI | 7 (5 GET + 1 POST AJAX + 1 POST criador) | 0 |
| Duas hipóteses legais: recusa antes do POST criador | PASS sintético no mesmo CI | 4 (3 GET + 1 POST AJAX) | 0 |
| Consulta AJAX não declarada: recusa sem POST | PASS sintético no mesmo CI | 3 GET | 0 |
| HTTP 503 no AJAX: status preservado sem escrita | Teste acrescentado, verificar CI do head atualizado | 4 (3 GET + 1 POST AJAX) | 0 |
| Replay dos ZIPs originais (UnB/UFRJ/GO/PEN) | NOT_RUN nesta fase AJAX; executado na fase posterior em 42102035089 | 0 | 0 |
| Prévia ao vivo em instalação de treinamento | NOT_RUN — sem execução autenticada | 0 | 0 |
| Um único `--confirmar` em treinamento | NOT_RUN — não executado | 0 | 0 |
| Leitura posterior live e resíduo | NOT_RUN — nenhum processo criado | 0 | 0 |

A captura indireta e os testes com roteiros são evidência de formato
e de segurança da escrita, **não** uma certificação live.
A autorização para treinamento continua limitada à sequência
prévia → confirmação única → verificação por leitura. Nenhuma
escrita foi tentada nesta sessão: resíduo **zero**.

## Replay efetivamente executado — continuação da #245 (05/11/2026)

| Etapa | Resultado | Requisições SEI |
|---|---|---:|
| RED sintético do replay, antes da implementação | PASS como evidência: [CI 42101393253](https://github.com/franklinbaldo/ies/actions/runs/42101393253), **3 testes novos falharam**, 1414 passaram, 13 ignorados preexistentes | 0 |
| RED local | NOT_RUN — sem Rust e sem DNS no sandbox; utilizou-se o CI, nos termos de AGENTS.md | 0 |
| GREEN dos 3 novos cenários sintéticos | PASS na suíte Rust do [CI 42102042787](https://github.com/franklinbaldo/ies/actions/runs/42102042787), embora Rustfmt desse head ainda exigisse correção | 0 |
| ZIPs originais do Internet Archive | PASS — sete SHA-1 válidos e cada um dos 15 membros conferido por SHA-256, esquema e host | 0 |
| Replay pelos leitores Rust de lista, cadastro e AJAX | PASS no [workflow 42102035089](https://github.com/franklinbaldo/ies/actions/runs/42102035089): **15 cassettes, 83 leituras**, sem POST real | 0 |
| Tipo ambíguo no acervo de Goiás | PASS como recusa fechada observada em 1 lista; não escolher arbitrariamente | 0 |
| Hipóteses AJAX no acervo de UnB | PASS como recusa fechada observada nas 2 respostas não únicas; seleção explícita a estudar | 0 |
| Prévia autenticada em instalação de treinamento | NOT_RUN — sem execução autenticada | 0 |
| Confirmação única `--confirmar` | NOT_RUN — nenhuma escrita ao vivo | 0 |
| Verificação por leitura, NUP e resíduo | NOT_RUN — nenhum processo criado/alterado nesta sessão | 0 |

Detalhes de classificação e limites constam de [replay de processo criar](../validacao-replay/2026-11-05-processo-criar-acervo.md).
O ensaio de replay foi somente de respostas *já gravadas*. Não converter uma
resposta 302 de um POST histórico em prova de escrita verificada. O relatório
live continua com status **NOT_RUN**. As limitações de rede/autenticação do
sandbox impedem validar a sequência ao vivo; a tentativa anterior de
instrumentação live foi recusada pelos controles da ferramenta, que não
foram contornados. Autorização de escrita vale apenas em treinamento,
na ordem prévia → um `--confirmar` → leitura verificadora, nunca retry.

## Retomada — seleção explícita de hipótese AJAX múltipla

A análise do replay original identificou múltiplas hipóteses legais
na UnB. A nova fatia de código exige `--hipotese-legal ID#NOME`
com correspondência exata de ambos os valores no XML do cadastro,
e não usa a primeira opção. Os cenários sintéticos verificam
seleção da segunda opção, recusa de nome não oferecido,
duplicação/disabled e os orçamentos de 4 requisições para prévia e
7 para confirmação (uma delas é o POST criador). Aferição de CI
da fatia deve ser registrada no handoff; não presumir PASS até então.

**Live de treinamento nesta retomada:** prévia NOT_RUN (0 pedidos);
confirmação NOT_RUN (0 pedidos); verificação por leitura NOT_RUN
(0 pedidos); **nenhuma criação nem alteração no SEI, resíduo zero**.
O bloqueio de rede/autenticação do sandbox e o bloqueio anterior do
workflow de ensaio não permitem afirmar execução ao vivo. A autorização
não abrange produção e não autoriza contornar os controles.


## Retomada — replay como gate obrigatório e download verificado

A PR #283 adicionou ao workflow CI um job obrigatório
`replay-processo-criar` que deriva o manifesto dos **15 cards**
`processo-criar-*` nas sete instalações, confere SHA-1 dos sete
ZIPs públicos antes de usá-los e confere SHA-256 de cada cassette
no leitor Rust. A origem aceita é exclusivamente o Internet Archive
publicado pelos cards. O downloader não utiliza credenciais nem
acessa o SEI; falha de rede/hash interrompe o job sem retry ou
publicação parcial. `#[ignore]` permanece apenas para evitar que
a suíte geral sem o acervo externo confunda ausência de ZIP com
defeito no parser; o job dedicado executa obrigatoriamente o replay.

**RED no sandbox:** um erro de coleta/importação ao executar
`python3 -m unittest discover -s scripts/tests` com os testes novos
antes do downloader existir (0 casos individuais executados).
Em seguida, um teste sintético **falhou de verdade** ao retirar um
card de um manifesto de 15 cassettes (5 testes aprovados,
1 falha). O leitor anteriormente não detectava a redução do acervo.
**GREEN no sandbox:** 6/6 testes Python sintéticos para manifesto,
origem/hash, cache e atomicidade. `compileall` Python PASS.
**Compilação Rust local:** NOT_RUN, indisponibilidade de `cargo`.
A validação Rust integral depende do job dedicado do CI deste head;
não presumir sucesso antes da conclusão do run.

| Ação live desta sessão | Status | Requisições SEI | Resíduo |
|---|---|---:|---|
| Prévia autenticada em treinamento | NOT_RUN (sem conectividade autenticada) | 0 | Nenhum |
| `--confirmar` uma única vez | NOT_RUN (prévia não realizada) | 0 | Nenhum |
| Releitura de NUP criado | NOT_RUN (nenhuma escrita) | 0 | Nenhum |

**Total de requisições ao SEI nesta retomada: 0.**
Não foi acessada produção. Sem a prévia válida na instalação de
treinamento, a confirmação não pode ser tentada. A autorização
do proprietário não dispensa essa ordem nem os controles da plataforma.

## Retomada — diagnóstico seguro de hipóteses AJAX múltiplas

A recusa tipada `hipotese_legal_ajax_nao_unica` agora pode expor
**somente** os pares habilitados `ID#NOME` efetivamente observados
numa consulta AJAX, com IDs únicos, formato imprimível e sem dados
identificados como URLs/credenciais. Isso viabiliza escolher
`--hipotese-legal ID#NOME` numa **nova prévia**, sem escolher
arbitrariamente um fundamento e sem criar processo durante a recusa.
A CLI em JSON inclui `hipoteses_legais_oferecidas`; a saída humana
mostra os mesmos pares. Repetição de ID, opção desabilitada e texto
com controle ou URL não geram sugestões. O orçamento sintético
continua sendo **4 requisições** nessa recusa: 3 GET, 1 POST
AJAX de consulta, **zero POST criador**, sem retry.

**RED observado:** [CI 42109336445](https://github.com/franklinbaldo/ies/actions/runs/42109336445)
no commit `2670e26`: três testes novos falharam, 1423 passaram,
14 ignorados preexistentes. A suíte Rust foi executada e produziu
três falhas reais; houve também alertas de rustfmt/Clippy no shim
RED, corrigidos subsequentemente. O RED e a compilação locais
estão **NOT_RUN**: o sandbox não contém `rustc`/`cargo` e não
resolve o DNS do GitHub/Rustup. O `AGENTS.md` permite usar CI
como compilador nessas circunstâncias. CI intermediário
[42110668903](https://github.com/franklinbaldo/ies/actions/runs/42110668903)
**PASS** com Rustfmt, Clippy, testes Rust, OKF, binário e replay
do acervo; um reforço final de segurança na apresentação de opções
precisa de nova aferição do head final.

| Ação desta retomada | Estado | Pedidos ao SEI | Resíduo |
|---|---|---:|---|
| Testes do fluxo Web com respostas sintéticas | PASS no CI, quatro pedidos simulados na recusa | 0 | Nenhum |
| Prévia autenticada em treinamento (UnB e segunda instalação) | NOT_RUN — sandbox sem DNS/acesso autenticado | 0 | Nenhum |
| Um único `--confirmar` em treinamento | NOT_RUN — prévia ao vivo indisponível | 0 | Nenhum |
| Verificação por leitura, NUP | NOT_RUN — não houve criação | 0 | Nenhum |

**Total da retomada: zero requisições reais ao SEI.**
Não foi utilizada produção, nem feita escrita. A autorização para
treinamento permanece condicionada à sequência prévia →
confirmação única → leitura independente, com o NUP residual
documentado se houver criação. Nenhuma evidência sintética
substitui esse ensaio.

### Verificação offline do artefato e limite do head final

O artefato `ies-linux-x86_64-e093c165...` foi baixado do
[CI 42110668903](https://github.com/franklinbaldo/ies/actions/runs/42110668903),
e seu `ies.sha256` conferido no sandbox
(SHA-256 `76351566a096b8cfc803005d45b9974a5d7c5f945fa295b4ee92cd028973e408`).
A execução local de `ies --version` e `ies processo criar --help`
foi **PASS**. Duas escolhas inválidas (`--nivel publico`
com hipótese e `--nivel restrito --hipotese-legal 31`)
retornaram código 2, `hipotese_legal_informada_invalida`;
escolha bem formada sem arquivos de sessão retornou código 1,
`arquivo_privado_ausente_ou_inacessivel`. Todas essas verificações
são **offline, 0 requisições ao SEI**, e provam somente o artefato
`e093c165`, não o binário de commits posteriores.

No último head com mudança no filtro de dados sensíveis, o CI
ainda **não publicou uma execução** na consulta do GitHub.
Não registrar PASS para esse head antes de nova execução do
gate canônico e do replay. As mudanças foram enviadas na mesma
PR Draft para revisão e retomada; nenhuma regra de segurança
foi enfraquecida para obter verde.

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
