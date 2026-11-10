---
type: ImplementationDecision
title: Leitura estrita da tela de atribuição de processo
operation: processo.atribuir
implementation: partial
status: adopted-for-scope
scope: prévia Web e confirmação CLI restrita à UnB de treinamento com POST único e releitura
selected_strategy: usuario-exato-da-lista-e-arvore-como-fonte
alternatives: usuario-exato-da-lista-e-arvore-como-fonte,substring,html-sem-conferencia,rest-wssei
evidence_level: pagina-real-todos-323eb01-testes-sinteticos
live_validation: inconclusive-unb-2026-11-09
---

# Atribuir processo — fatia de leitura (Refs #264)

A página de atribuição observada na UnB usa `frmAtividadeAtribuir` com
`selAtribuicao`, opção `null` para remover, `hdnIdProtocolo` e
`sbmSalvar`. O `select` não marca quem está atribuído: esse dado está na
situação `aberto_em` do nó do processo, específica da unidade.

## Alternativas consideradas

- `usuario-exato-da-lista-e-arvore-como-fonte` (escolhida): login inteiro
  na lista `selAtribuicao`; o atribuído atual vem de `aberto_em` na árvore.
- `substring`: casar parte do nome ou do login. Descartada: ambígua.
- `html-sem-conferencia`: enviar o formulário sem conferir action,
  identidade e botão. Descartada: sem prova de identidade.
- `rest-wssei`: fora do escopo Web do IES.

## Comparação

Só a escolhida evita ambiguidade e não depende de um estado que o
`select` não publica.

## Decisão

A primeira fatia implementa um leitor HTML estrito, reutilizando
`html_atributos` e `localizar_formulario`, sem executar JavaScript.
Conferir um único formulário POST, `action` assinado do processo aberto,
único botão Salvar, campo oculto de identidade, lista simples, opção
`null` habilitada e não ambígua. Escolher usuário por **login inteiro**,
sem buscar por substring ou nome, e recusar estado não observado ou
atribuição já aplicada. A função de planejamento é pura e não envia POST.

A prévia Web já usa esse leitor. Nesta fatia, ela também deriva a submissão
com o primitivo único `formulario_web`, exigindo que os ocultos servidos,
o botão Salvar e a opção `selAtribuicao` produzam exatamente o corpo do
navegador. Rejeita um mesmo ID de usuário associado a dois logins. Os
testes sintéticos conferem o payload de atribuição e o `null` de remoção,
sem requisições. A prévia foi extraída para um fluxo injetável com testes
 que afirmam 4 GET e 0 POST, e a CLI preserva status HTTP e mensagem saneada.
O fluxo interno de transporte agora envia um POST no máximo uma vez,
reabre a árvore e compara a atribuição na unidade e no processo originais.
Falha do POST não gera retry. Se a releitura não comprovar exatamente o
pedido, a operação devolve efeito incerto (saída 3). A revisão da PR #316
identificou um risco adicional: **releitura coincidente depois de HTTP 403
ou validação da Infra não prova que o nosso POST escreveu**; outro agente
pode ter modificado o processo no intervalo. A resposta recusada não
pode ser reconciliada como sucesso, mesmo com o estado esperado. O mesmo
vale para 302 sem ação e identidade compatíveis. O 302 para
`arvore_visualizar` **também** precisa do `id_procedimento` explícito;
não basta indicar a ação. Essa exigência protege atribuir e remover de
falso sucesso por ação concorrente, conforme RED de duas regressões
adversariais no [CI 42214490209](https://github.com/franklinbaldo/ies/actions/runs/42214490209)
(**2 FAIL novos, 1.716 PASS, 23 ignored existentes**) e observação
do destino `ACEITE` com ID nos testes do Todos @323eb01. Apenas uma resposta
aceita e **identificada** (302 para ação, processo e origem esperados)
pode ser confirmada pela releitura; não há nova escrita nem navegação
pelo Location. Uma **falha de rede sem resposta não prova autoria**,
mesmo que a árvore relida coincida: outro agente pode ter feito a mesma
atribuição/remoção. Conserva-se a falha original, o estado relido como
possível resíduo e a saída 3, sem retry. Esta regra corrige o falso
sucesso apontado na revisão 🦅 de #316 (head `72bab3b`). O 302 do aceite observado no Todos @323eb01 também publica
`acao_origem=procedimento_atribuicao_cadastrar`. Não basta a ação de
destino com o ID correto: um 302 sem ação de origem não vincula o POST
à leitura coincidente, que pode ter resultado de outro agente. O RED
[42267049782](https://github.com/franklinbaldo/ies/actions/runs/42267049782)
demonstrou dois falsos sucessos (atribuir/remover): **2 FAIL novos,
1.739 PASS, 23 ignored existentes**, com seis tentativas por caso.
Agora a falta de origem resulta em saída 3, sem segundo POST nem seguir
Location. Não se aceita origem diferente ou repetida.
Uma resposta
HTML 200 genérica também não é prova do POST, mesmo quando a árvore
coincide: a escrita pode ter sido feita por outro agente. O teste-first
[RED 42257965685](https://github.com/franklinbaldo/ies/actions/runs/42257965685)
reproduziu o falso sucesso na atribuição e na remoção (**2 FAIL novos,
1.737 PASS, 23 ignored preexistentes**), com 4 GET prévios, 1 POST e
1 GET independente em cada caso. A correção conserva efeito incerto
(saída 3), sem retry, até existir aceite observado para HTML 200.
Os testes injetados afirmam 5 GET e 1 POST (6 tentativas) a frio, tanto
em atribuição quanto em remoção, e as regressões de concorrência exigem
a mesma contagem exata. A confirmação pela CLI está restrita à origem exata da UnB de treinamento;
o ensaio ao vivo de 09/11/2026 terminou com efeito incerto (saída 3)
e possível resíduo, sem comprovação causal. O replay específico offline
continua obrigatório no CI.
Não confundir a cobertura sintética com autorização para enviar.


A reconciliação também precisa controlar a **falha da própria leitura**
após o POST. O RED [42222330393](https://github.com/franklinbaldo/ies/actions/runs/42222330393)
(2 FAIL novos, 1.718 PASS, 23 ignored) demonstrou que HTTP 503 na
árvore guardada levava o leitor a tentar outra rota em vez de terminar
a única releitura de prova. Optou-se pela correção mínima da leitura Web:
5xx é falha efetiva do servidor, não indício de link expirado; propagar
o erro tipado imediatamente, preservando 403 como possível link
envelhecido. Na atribuição, a projeção de erro usa status e mensagem
saneada da releitura quando a tentativa de POST não publicou HTTP.
Alternativa rejeitada: repetir a abertura por outra rota após 503,
porque violaria o orçamento de uma verificação e obscureceria o diagnóstico.
A escrita permanece fechada nas origens não autorizadas; na UnB de treinamento,
o POST único de 09/11/2026 produziu saída 3 e não autoriza inferir sucesso.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-atribuir.md)
- [Evidências da matriz](../../contracts/evidencias/processo-atribuir.md)
- [Teste da trava de escrita](../../tests/cli_processo_atribuir.rs)
- [Testes de forma e payload](../../src/processo_atribuir/tests.rs)
- [Testes de transporte da prévia](../../src/processo_atribuir/tests_fluxo.rs)


[Formulário real no Todos](https://github.com/franklinbaldo/todos/blob/323eb01/tests/paginas_reais/processo_procedimento_atribuicao_cadastrar.html),
[casos de teste](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_ciclo_atribuir_010.py)
e [cassette UnB](../../cassettes/unb/processo-atribuir-unb--bf0d4327d7d9.md).
Os testes Rust são sintéticos e documentam a origem acima de cada caso
derivado; não comprovam POST nem efeito no SEI.

## Limitações e revisão

`--confirmar` recusa `origem_fora_do_treinamento` antes de qualquer
rede em origens que não são [instalações públicas de treinamento](../../docs/interfaces/credenciais-de-treinamento.md).
O fluxo injetável possui testes de POST único,
releitura e saída 3. Prévia em duas instalações de treinamento já
comprovada: UnB (atribuição) e [GO (`--remover`, 5 requisições)](../validacao-live/2026-11-06-processo-atribuir-go.md). A decisão segue provisória: a confirmação ao vivo teve efeito incerto,
e a remoção em treinamento ainda não foi executada. O RED observou 4 testes falhando,
1.689 aprovados no [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454).

## Controles efetivamente acionáveis (06/11/2026)

Uma lista e um botão presentes no HTML podem não ser oferecidos pela
interface Web: `hidden` no próprio formulário, no botão Salvar ou no
`select`, e `fieldset disabled`, tornam a ação inerte. Em vez de
derivar um POST de campos estruturalmente válidos mas inacessíveis,
recusar a tela com `ErroHtml` antes de qualquer escrita. O leitor
inspeciona somente atributos via `html_atributos`; nenhum JS é executado.
Teste sintético com RED antes da mudança e cenário de transporte para
quatro GET de prévia e nenhum POST. A CLI `--confirmar` segue fechada nas demais origens; na UnB de treinamento,
o ensaio posterior teve resultado inconclusivo.

## Revisão da execução CLI em treinamento (09/11/2026)

A #264 exige a confirmação pela **CLI real**, e não apenas um POST
independente da bancada. A mesma operação injetável é chamada pela CLI
para o `--confirmar`, sem duplicar submissão ou parser. A autorização
operacional fica estritamente condicionada à origem exata
`https://sei.treinamento.unb.br/sei`: é onde o oráculo Todos registra
um 302 identificado. Qualquer outra origem é recusada antes da rede,
inclusive produção, sem criar subcódigo.

O E2E da PR #547 executa a CLI real por HTTPS em **loopback** contra o
anti-IES e o SQLite efêmero, depois invoca outra vez `ies processo NUP`
na mesma bancada para observar a alteração ou remoção. Sua execução
sintética não autoriza afirmar paridade dos efeitos de POST das cassettes
nem sucesso em treinamento externo. A confirmação remota recebeu em 09/11/2026 um ensaio controlado na UnB,
com prévia válida, POST único, saída 3 e estado posterior observado, mas
sem prova causal. A remoção remota segue **NOT_RUN**. A ampliação para outras
instalações depende de observação e revisão independentes.

## Evidência que altera a decisão (09/11/2026)

O [relatório de treinamento da #264](../validacao-live/2026-11-09-processo-atribuir-unb-post-incerto.md) registra uma confirmação pela CLI Rust, na origem UnB de treinamento, após duas prévias válidas. O POST foi único; a releitura encontrou `aluno2`, mas a CLI devolveu **saída 3** por falta de prova causal da alteração. A [PR #605](https://github.com/franklinbaldo/ies/pull/605) foi integrada com [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42703581460).

**Decisão preservada:** não declarar sucesso pela mera coincidência entre estado pedido e estado relido. Não repetir o POST incerto, não alegar resíduo limpo e não realizar remoção como suposta prova retroativa. A compatibilidade sintética IES↔anti-IES↔SQLite permaneceu PASS anterior; as respostas 302 dos POSTs históricos não trazem o ID decimal singular exigido para confirmação estrita, de modo que o confronto causal com cassette permanece pendente. Nenhum teste live de remoção foi executado.
