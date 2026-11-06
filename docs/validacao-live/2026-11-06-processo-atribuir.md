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
