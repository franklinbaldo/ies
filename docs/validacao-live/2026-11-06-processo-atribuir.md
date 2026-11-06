---
type: SEIObservation
title: Validação de processo atribuir — transporte sintético e prévia UnB
description: Ensaio de prévia autenticada UnB sem escrita e transporte sintético com POST único
---

# processo atribuir — 06/11/2026

PR #316, issue #264. O fluxo injetável deriva o payload observado no
formulário Web e testa a sequência de **um POST e uma releitura nova**,
sem retry. Em bancada sintética, cada confirmação custa 5 GET + 1 POST
(6 tentativas). Há cobertura de atribuição por login exato, remoção com
`null`, falha de transporte no POST com releitura conclusiva e inconclusiva.
A CLI pública preserva a trava de segurança de `--confirmar` **antes de
abrir sessão**. RED no [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454):
**4 testes novos FAIL, 1.689 PASS e 22 ignored preexistentes**.

## Replay específico do acervo (offline)

O teste `processo_atribuir::acervo::replay_de_processo_atribuir_no_acervo`
usa o **leitor real** `ler_tela` com as respostas históricas de sete
instalações, depois de conferir SHA-1 dos sete ZIPs, SHA-256 de cada
cassette, `schema_version`, completude e origem autorizada. O CI
[42201836542](https://github.com/franklinbaldo/ies/actions/runs/42201836542)
classificou **42 respostas**: **28 formulários GET aceitos** (quatro em
cada instalação), **zero recusas**, e **14 respostas POST marcadas como
não aplicáveis à prova de escrita**. Não se inferiu atribuição aplicada
do retorno histórico do POST. O replay utilizou **zero requisições reais**.

| Ensaio | Instalação | Estado | GET | POST | Resíduo |
| --- | --- | --- | ---: | ---: | --- |
| Prévia Web | UnB treinamento | **PASS** | 4 | 0 | nenhum |
| Prévia Web | UFRJ treinamento (dois NUPs distintos) | **FAIL** | não observado | 0 de atribuição | nenhum |
| Confirmação única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Remoção única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |

**Ensaio UnB, 06/11/2026:** [GitHub Actions 42205702494](https://github.com/franklinbaldo/ies/actions/runs/42205702494), conta pública da instalação de treinamento. Login **PASS, saída 0, 5 requisições**; após intervalo superior a 30 segundos, prévia `processo atribuir 23106.001559/2026-11 --usuario aluno2` **PASS, saída 0, 4 GET e 0 POST de atribuição**. `confirmado=false` e `alterado=false`. Valores de cookie, credencial, URL assinada e `infra_hash` permaneceram em arquivos privados do executor; apenas código de saída e contagem foram publicados. A prévia NÃO comprova escrita. Não houve confirmação, remoção, retry de escrita, resíduo nem acesso a produção.

A segunda instalação foi ensaiada depois, mas a prévia **falhou** em duas observações distintas (detalhes abaixo). Confirmação única e remoção continuam **NOT_RUN** porque `--confirmar` permanece fechado antes da rede. A capacidade continua incompleta: falta diagnosticar a divergência na UFRJ, obter prévia válida ali e, só então, confirmar/remover uma única vez em treinamento com prova por releitura e resíduo registrado.

## Regressão de destino 302 genérico

Observação de segurança: `acao=principal` não identifica, por si só,
o processo atribuído. A leitura coincidente após esse 302 pode ter sido
produzida por outro agente. O teste sintético foi introduzido antes
da correção, e o [CI 42204843683](https://github.com/franklinbaldo/ies/actions/runs/42204843683)
registrou **1 FAIL novo, 1.706 PASS e 23 ignored preexistentes**:
o fluxo marcava `confirmado=true` com 4 GET de prévia, 1 POST e 1 GET de
verificação, embora a resposta não identificasse o processo. A correção
exige `id_procedimento` explícito em redirects para `principal`;
rejeição continua com saída 3, sem segundo POST. Um destino `principal`
com ID correto permanece aceito e é testado; sem tráfego real nesta etapa.

## Regressão de destino `arvore_visualizar` genérico

A revisão 🦅 do head `0fc1922` identificou que
`acao=arvore_visualizar` sem `id_procedimento` ainda era aceito.
Um segundo agente poderia produzir a atribuição ou remoção desejada
entre o POST e o GET de verificação, causando falso sucesso.
O RED [CI 42214490209](https://github.com/franklinbaldo/ies/actions/runs/42214490209)
comprovou o defeito em **dois testes novos** (atribuir/remover):
**2 FAIL, 1.716 PASS, 23 ignored existentes**, após **4 GET de
prévia + 1 POST + 1 GET de releitura** por cenário.
O fluxo agora exige ID explícito também para `arvore_visualizar`.
Os testes de sucesso usam a forma `ACEITE` observada no Todos @323eb01,
com ação, origem e ID do mesmo processo. Os destinos genéricos continuam
efeito incerto (saída 3), sem repetir POST ou seguir o 302.
**Live desta fatia: NOT_RUN**, 0 requisições reais e 0 resíduos.
A prévia autenticada da UnB registrada acima permaneceu a última
prévia bem-sucedida desta fatia; a confirmação pública segue travada.

## Releitura com HTTP 503 após POST

Observação de bancada, **sem acesso ao SEI**: quando a releitura da
árvore após o POST recebe HTTP 503, o leitor do índice tratava a resposta
como link envelhecido e tentava outra rota. A saída de erro da atribuição
também não expunha o HTTP da leitura, porque consultava somente a falha
do POST. Dois testes adversariais foram introduzidos **antes da correção**.

O RED [CI 42222330393](https://github.com/franklinbaldo/ies/actions/runs/42222330393)
registrou **2 FAIL novos, 1.718 PASS, 23 ignored preexistentes**:
os dois falharam com `requisição além do roteiro` após a recusa 503 da
única releitura prevista. A correção evita o caminho alternativo quando
o servidor respondeu 5xx e conserva o status/mensagem da verificação
quando a escrita já tiver sido tentada. Testes exigem **4 GET de prévia,
1 POST e somente 1 GET de verificação**, saída 3, mensagem saneada e
zero retry, tanto com 302 esperado quanto com falha de rede do POST.

**Live nesta correção de HTTP 503:** NOT_RUN; 0 GET reais, 0 POST reais,
0 escritas e 0 resíduos. A prévia UnB histórica permaneceu a única
prévia autenticada bem-sucedida até o ensaio UFRJ documentado a seguir.

## Nova observação: duas prévias UFRJ recusadas antes de escrever

Instalação: **UFRJ 4.1.5 de treinamento**, conta pública `servidor1`
e unidade PR-P, exclusivamente via CLI Web. O executor manteve
origem, senha e cookie em arquivos privados `0600`, não publicou
URLs assinadas nem `infra_hash` e respeitou intervalos de 35 segundos.
Os workflows temporários foram removidos do branch após os ensaios.

- [Run 42229686084](https://github.com/franklinbaldo/ies/actions/runs/42229686084):
  login **PASS** (saída 0, **6 requisições**). Prévia de **remoção**
  em `99079.900901/2026-02`: **FAIL**, saída 1,
  `situacao_da_atribuicao_nao_observada`; `requisicoes=null`
  na saída de erro (quantidade não observada). Nenhum POST de atribuição.
- [Run 42230157551](https://github.com/franklinbaldo/ies/actions/runs/42230157551):
  novo login **PASS** (saída 0, **6 requisições**).
  Leitura independente de **outro NUP**, `99079.901790/2026-43`,
  **PASS** (saída 0, **6 requisições**), com PR-P presente em
  `aberto_em` e `atribuido_a=null`. Após 35 segundos, prévia
  de atribuição para o login `servidor1` no mesmo segundo NUP:
  **FAIL**, saída 1, `situacao_da_atribuicao_nao_observada`,
  `requisicoes=null` na saída de erro. Nenhum POST de atribuição.

O job de cada workflow encerrou com `success` porque registrava
a recusa sem encerrar o diagnóstico com erro; **esse verde do job
não torna a prévia PASS**. Os dois processos são diferentes: não
houve repetição da prévia recusada sobre o mesmo processo.
O comando de leitura comum publicou PR-P e ausência de atribuição
no segundo NUP, enquanto a prévia de atribuição não conseguiu
identificar sua situação. Há divergência a investigar entre a unidade
ativa da tela e a unidade publicada pela árvore, ou mudança concorrente
entre as observações; **nenhuma causa foi comprovada**. Não converter
`null` em ausência de requests, nem a recusa em sucesso.

**Saldo desta sessão:** login UFRJ PASS em duas sessões (6+6 pedidos);
uma leitura adicional PASS (6 pedidos); duas prévias FAIL
(contagens por comando **não observadas**); **0 POST de atribuição**,
**0 confirmações**, **0 alterações**, **0 resíduos**, **0 retry de escrita**.
A segunda instalação **não satisfaz** o critério de prévia PASS.
Próxima fatia: confrontar offline os fatos da tela/árvore UFRJ,
estabelecer um teste de regressão antes da correção e só voltar ao
treinamento com o orçamento controlado. Enquanto isso,
`--confirmar` fica recusado antes da sessão e da rede.

## Diagnóstico offline da recusa UFRJ (fatia posterior)

Os dois ensaios UFRJ acima **não** publicaram a sigla da unidade da
tela de abertura, apenas a presença da unidade PR-P na leitura comum
da árvore; portanto **a causa ainda não foi identificada**. A bancada
reproduz a hipótese de divergência com uma tela sintética de unidade
`TESTE` e árvore com `PR-P`. O teste verifica que não há inferência
de atribuição nem POST: **3 GET e 0 POST** (a recusa ocorre antes do
GET do formulário). Essa é uma hipótese diagnóstica, não uma afirmação
sobre a unidade verdadeira da sessão UFRJ.

A recusa tipada `situacao_da_atribuicao_nao_observada` preserva
`unidade` (da tela), `aberto_em` (unidades da árvore ou `null` quando
não lidas), `motivo` e `requisicoes` tentadas até a recusa. Isso
permite observar a discrepância com a próxima prévia em processo
**diferente** e com orçamento controlado, sem publicar cookies nem
URLs assinadas, e sem relaxar a guarda. O teste foi enviado primeiro
como RED no commit `9a3b964`; a execução RED e o CI do head novo
precisam ser confirmados no GitHub. Não houve nova execução live,
nem escrita, nem resíduo nesta fatia (**NOT_RUN, 0 GET/POST reais**).


## Retomada 🦫 — contagem pública após POST incerto e merge da main

A observação anterior permanece a mesma: na UFRJ de **treinamento**
houve duas prévias **FAIL**, com contagens de requests **não observadas**
nos logs antigos; a causa da discrepância entre unidade da tela e árvore
continua **não comprovada**. Não repetimos esses ensaios.

O fluxo interno de atribuição já contava as tentativas em
`ProcessoAtribuido.requisicoes`, mas descartava a contagem quando o
POST tinha sido tentado e a releitura devolvia
`efeito_incerto_na_atribuicao` (saída 3). A regressão sintética foi
adicionada primeiro no commit `91862af`: recusa expressa HTTP 403,
árvore alterada concorrentemente por outro agente, **4 GET de prévia +
1 POST + 1 GET de prova = 6 tentativas**, sem retry e sem falsa
confirmação. O JSON da falha agora inclui `requisicoes: 6` além da
causa/status, sem divulgar URL assinada ou cookie. A leitura real
desse resultado ainda precisa de CI; **RED do commit de teste:
NOT_RUN**, pois nenhum workflow foi localizado para aquele SHA.

A `main` foi incorporada por merge de dois pais no commit `ec2d21b`
(sem rebase ou force-push). O workflow de replay manteve o cache dos
retornos já integrado à main, adicionou o sétimo ZIP ao manifesto com
SHA-1 verificado e conservou o replay obrigatório #264; a árvore
sintetizada foi comparada com a árvore da main sem alterações
inesperadas. **CI do novo head: ainda não comprovado**. Não declarar
PASS, GREEN ou prontidão de integração enquanto os gates não rodarem.

**Live nesta retomada: NOT_RUN**, 0 GET reais, 0 POST reais,
0 escritas, 0 resíduos, nenhum retry. Prévia UnB histórica permanece
PASS (4 GET), UFRJ permanece FAIL (duas prévias). O `--confirmar`
público continua fechado **antes de abrir sessão ou rede**. Para
concluir #264 ainda faltam CI no head atual, prévia PASS numa segunda
instalação de treinamento e confirmação/remoção únicas verificadas
por leitura, com registro dos efeitos e resíduos.


## Regressão: HTML 200 genérico após POST não é aceite identificado

Observação **antes do RED**: o oráculo Todos
`tests/test_ciclo_atribuir_010.py @323eb01` registra aceite Web
da UnB via **302 com `arvore_visualizar` e ID explícito**.
O replay histórico deste projeto classificou 14 respostas POST como
**insuficientes para provar escrita**. Uma resposta HTML 200 genérica,
sem validação da Infra e sem marcador positivo conhecido, não vincula
a alteração posteriormente observada ao nosso POST. Na conta
compartilhada, outro agente pode ter alterado a atribuição durante
a releitura. Não se inferiu da observação que todo HTML 200 seja recusa;
o resultado é **efeito incerto**.

**RED test-first**: commit `deee836`,
[CI 42257965685](https://github.com/franklinbaldo/ies/actions/runs/42257965685),
suíte Rust **1.737 PASS / 2 FAIL novos / 23 ignored
preexistentes**. Os dois FAIL mostraram que o fluxo anunciava
`confirmado=true` tanto para atribuir quanto para remover após
HTML 200 genérico e mudança sintética concorrente. A correção exige
aceite identificado antes de reconciliar. Cada cenário usa exatamente
**4 GET de prévia + 1 POST único + 1 GET de prova = 6 requisições**,
sem retry. O resultado esperado agora é `efeito_incerto_na_atribuicao`,
**saída 3**, ainda que a árvore coincida. Não foi alterado o tratamento
de 302 com identidade do mesmo processo nem da falha de rede sem
resposta do POST. Nenhum teste antigo foi desligado ou enfraquecido.

**Live nesta fatia: NOT_RUN**, 0 GET reais, 0 POST reais,
0 escritas e 0 resíduos. UnB: prévia histórica PASS (4 GET).
UFRJ: duas prévias históricas FAIL (requisições dos comandos
não observadas). A causa UFRJ continua a verificar. O
`--confirmar` público permanece bloqueado antes da sessão/rede;
não houve tentativa de escrita nem repetição de ensaio falho.

## Regressão: 302 sem origem da atribuição não prova autoria

**Observação antes do RED:** o caso de aceite Web do Todos
`tests/test_ciclo_atribuir_010.py @323eb01` publica, além de
`arvore_visualizar` e `id_procedimento`, a ação de origem
`procedimento_atribuicao_cadastrar`. Sem esse vínculo, mesmo a
releitura coincidente pode retratar uma escrita concorrente.
A forma sem `acao_origem` não foi observada como aceite.

**RED:** commit `d7af6f3`,
[CI 42267049782](https://github.com/franklinbaldo/ies/actions/runs/42267049782),
**1.739 PASS / 2 FAIL novos / 23 ignored preexistentes**.
Nos dois testes (atribuir e remover), o código anunciava aplicação após
302 com ID correto mas sem ação de origem e releitura alterada por outro
agente. A regressão exige efeito incerto (saída 3), exatamente
**4 GET de prévia + 1 POST + 1 GET de verificação = 6 requisições**,
sem seguir Location nem repetir POST.

**Correção:** a ação de origem deve constar explicitamente e ser única;
a falta dela não se concilia como sucesso. As respostas positivas com
ação, ID e origem conferidos permanecem cobertas pelos testes existentes.
**Live nesta fatia: NOT_RUN**, 0 GET, 0 POST, 0 escritas e 0 resíduos.
Prévia UnB histórica PASS (4 GET); duas prévias UFRJ permanecem FAIL
(contagens não observadas), sem prova da causa. Não repetidas.
`--confirmar` público segue fechado antes da sessão/rede.
