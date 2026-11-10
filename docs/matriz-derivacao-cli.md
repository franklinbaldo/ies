---
type: Guide
title: Matriz de derivação das capacidades da CLI IES
description: Cruzamento dos comandos declarados, código executável, evidências e lacunas reais
---

# Matriz de derivação da CLI — implementar antes de ampliar

**Base de derivação:** `main@52be0e1`, auditada em 04/11/2026 após as PRs #193 e #206; revalidar o SHA, o despacho e os checks a cada rodada. Propostas fora da `main`
não são capacidades aprovadas. Fontes: [superfície Clap e despacho](../src/main.rs),
[contrato público](../contracts/cli.md), [matriz de evidências](../contracts/evidencias/README.md),
[política de acesso](interfaces/acesso-por-instalacao.md) e
[roadmap #56](https://github.com/franklinbaldo/ies/issues/56).

## Regra de priorização

**Primeiro entregar o que a CLI já promete.** Não criar novos verbos,
subcomandos, opções, modos opt-in ou variantes de replay apenas para substituir
um caminho já declarado e ainda sem implementação real. Cada PR de capacidade
deve identificar o comando **existente**, seu caminho no despacho, o resultado
atual, a lacuna eliminada e o ensaio que a demonstra. Mudanças fundamentais
de sintaxe exigem proposta antes/depois e decisão do proprietário; esta matriz
não altera o contrato público nem impede correções de segurança necessárias.

**Estados distintos:**
- **STUB**: caminho reconhecido, mas cai em `nao_implementado` ou recusa
  expressamente a operação remota pedida.
- **PARCIAL LOCAL**: opera arquivos/capturas privadas ou prévias, sem
  execução remota da capacidade de domínio.
- **PARCIAL WEB**: há transporte Web limitado no código, mas sem atender
  todo o contrato. Não é sinônimo de validação live.
- **LOCAL**: operação local de infraestrutura realmente implementada;
  não é uma capacidade SEI Web autenticada.
- **PENDENTE PR**: proposta fora da `main`; **não** computar como entregue.

## Fatias WIP de escrita do processo

As notas de handoff das fatias em andamento (#247, #257, #258, #259, #269)
estão na [matriz das fatias WIP](matriz-derivacao-cli-handoffs.md).

## Matriz: 20 caminhos principais já expostos

| Comando existente | Estado na main | Já demonstra | Falta implementar, sem novo comando/flag | Referência |
| --- | --- | --- | --- | --- |
| `ies processo NUP` | **PARCIAL WEB** (#59 cumprida) pela árvore; `--resumo` e as opções de captura privada aposentados em 04/11/2026 (decisão do proprietário) | Abre pelo caminho mais barato do índice (link da árvore 1, link colhido do Controle 2, entrada por id 3) ou pela pesquisa rápida/linha do Controle (3–4); documentos com pastas, movidos (também de volta), ícone e cancelamento, anexados, pai do anexado, onde está aberto ou sobrestado (0 req.), andamentos, propriedades e marcadores em lista (custos nos contratos); o Controle lido é colhido no índice | Live PASS UnB, GO, PEN1, UFRJ 4.1.5, PEN3 e PEN4 5.0.4 (frio 5 req., quente 4; [varredura](validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)); propriedades e movidos: UnB/PEN1; marcadores: UnB, UFRJ, PEN3, PEN4; [replay do acervo](validacao-replay/2026-11-04-processo-obter-acervo.md) | [contrato](../contracts/cli/processo-obter-arvore.md), [andamentos](../contracts/cli/processo-obter-andamentos.md), [pastas](../contracts/cli/processo-obter-pastas.md), [movidos](../contracts/cli/processo-obter-movidos.md), [propriedades](../contracts/cli/processo-obter-propriedades.md), [marcadores](../contracts/cli/processo-obter-marcadores.md), [situação](../contracts/cli/processo-obter-situacao.md), [#59](https://github.com/franklinbaldo/ies/issues/59) |
| `ies processo buscar TERMO [--pagina N]` | **PARCIAL WEB** (#58, núcleo em main) | Pesquisa por protocolo pela sessão de `ies login`: link `protocolo_pesquisar` do Controle, GET da tela, POST derivado; 302 → `processo_unico` (só o id), lista com `total_declarado` e `faixa_observada` (separador de milhar), `sem_resultado` (saída 0); `--pagina N` por `navegar()`; linha com acerto em documento (GO) publica só o processo; 403 da borda é `bloqueio_de_seguranca_da_instalacao`. Página 1: GET + GET + POST; página N: + 1 POST. Replay do acervo: 52 cassettes nas 7 instalações | Live: PASS UnB, GO, PEN3 (NUP fora da caixa localizado), UFRJ 4.1.5 (dentro e fora da caixa, lista, `--pagina 2`) e PEN4 (dentro e fora da caixa, `sem_resultado`; [varredura](validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)); lista da PEN4 FAIL → corrigida offline (processo visitado com `class` repetido diferente), ao vivo NOT_RUN; NUP gerado no dia sai `sem_resultado` na UFRJ; PEN1 sem índice, PEN2 NOT_RUN; linhas a menos que a faixa escrita na barra (GO 5.0.3, 9 de 10) saem com `linhas_ausentes_da_faixa`, sem afirmar completude; termo só de dígitos ao vivo PASS na PEN3 (o 403 do Cloudflare veio no GET do Controle, não pelo termo); filtros além do protocolo | [contrato](../contracts/cli/processo-buscar.md), [replay](validacao-replay/2026-11-04-processo-buscar-acervo.md), [live PEN3](validacao-live/2026-11-04-processo-buscar.md), [#58](https://github.com/franklinbaldo/ies/issues/58) |
| `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]` | **PARCIAL WEB** (#173); `--confirmar`: escrita remota | Prévia: abertura + GET da tela "Enviar Processo" + POST da consulta do autocomplete declarada nela (leitura; destino pela sigla exata, ausente/repetido/já aberto recusados; sigla com `/`: até 4 consultas na ordem do Todos, sigla e termo alargado no órgão do prefixo e no da tela); `--confirmar`: um POST (`hdnUnidades` = item escolhido; `--manter-aberto` marca a caixa) e verificação pelo destino e pela origem em `aberto_em`; escrita não confirmada sai com 1, sem retry; a frio 4 GET + 1 POST / 5 GET + 2 POST | Prévia live PASS UFRJ 4.1.5 (4 req.); GO 5.0.3 PASS (`CBM/GECONT-18149`, `DETRAN/GECONT-18166` na 1ª consulta, 4 req.; inexistente recusado) ([live](validacao-live/2026-11-04-processo-enviar-go.md)); [replay](validacao-replay/2026-11-04-processo-enviar-acervo.md) 25 envios verificados; escrita live PASS UFRJ 4.1.5 (PR-P → PR-A, 6 req. quente cada; com `--manter-aberto` `depois` [PR-A, PR-P]; sem ele a origem sai, `depois` [PR-A]; leituras independentes conferem) ([escrita](validacao-live/2026-11-04-processo-enviar-escrita.md)); escrita live PASS UnB 4.0.9 (Unidade1 → Unidade2) e PEN1 5.0.4 (TESTE → TESTE_1_1), mesmo padrão, prévia a frio 5 e `--confirmar` 6 req., leituras independentes conferem ([UnB e PEN1](validacao-live/2026-11-05-processo-enviar-escrita-unb-pen1.md)); vários destinos/grupos, retorno programado não cobertos | [contrato](../contracts/cli/processo-enviar.md), [decisão](decisoes/processo-enviar.md), [live](validacao-live/2026-11-04-processo-enviar.md) |
| `ies processo anexar NUP --a NUP_PRINCIPAL [--confirmar]` | **PARCIAL WEB** (#270; PR #278) | Prévia: abre principal, lê tela `procedimento_anexar`, identifica anexado por consulta `protocolo_RI1023`, confere assinaturas por `procedimento_anexacao_verificar`; recusas por NUP exato e identidade duplicada; confirmação bloqueada antes de rede | RED 8 testes no CI #42084762362; live/replay NOT_RUN (0 requisições); falta POST único + verificação por leitura e escrita não confirmada 3 | [contrato](../contracts/cli/processo-anexar.md), [decisão](decisoes/processo-anexar.md), [live](validacao-live/2026-11-05-processo-anexar-handoff.md) |
| `ies processo concluir NUP [--confirmar]` | **PARCIAL WEB** (#172) | Prévia só com a abertura (2–3 req.); uma escrita pela forma da barra: tela "Conclusão de Processo" + POST (4.1/5.0) ou GET de `concluirProcesso()` (4.0); verificação por `aberto_em` numa leitura nova; escrita não confirmada sai com 1, sem retry; a frio 5 GET + 1 POST / 6 GET | Live PASS UFRJ 4.1.5 (`tela`) e UnB 4.0.9 (`funcao`), requisições iguais ao orçamento; [replay](validacao-replay/2026-11-04-processo-concluir-acervo.md) 219 escritas verificadas; PEN1 5.0.4 não observada; concluir com reabertura programada, vários processos | [contrato](../contracts/cli/processo-concluir.md), [decisão](decisoes/processo-concluir.md), [live](validacao-live/2026-11-04-processo-concluir.md) |
| `ies processo atribuir NUP --usuario LOGIN` / `--remover` | **PARCIAL WEB** (#264) | Prévia (4 GET): árvore, formulário, login exato e recusa de ambiguidade; fluxo injetável: POST único + releitura (6 req. no cenário a frio), 503 na prova preservado e saída 1 sem retry; `--confirmar` CLI ainda bloqueado antes da sessão | Replay obrigatório das 7 instalações PASS (42 respostas); prévia UnB atribuir PASS (4 GET), prévia GO remover PASS (5 req. + 6 no login); atribuição/remoção efetivas e releituras live NOT_RUN; RED HTTP503 2 FAIL / 1.718 PASS [run](https://github.com/franklinbaldo/ies/actions/runs/42222330393) | [contrato](../contracts/cli/processo-atribuir.md), [decisão](decisoes/processo-atribuir.md), [live](validacao-live/2026-11-06-processo-atribuir.md) |
| `ies processo reabrir NUP [--confirmar]` | **PARCIAL WEB** (#172) | Prévia só com a abertura; `reabrirProcesso()` da barra → GET da tela do nó → URL `procedimento_reabrir` (corpo da função no 4.0, `var linkReabrirProcesso` no 4.1/5.0) → um GET (a escrita); verificação pela unidade da sessão em `aberto_em`; escrita não confirmada 3, sem retry; a frio 6 GET | Live PASS UFRJ 4.1.5 (`variavel`) e UnB 4.0.9 (`funcao`), 5 req. cada; [replay](validacao-replay/2026-11-04-processo-reabrir-acervo.md) 11 reaberturas verificadas; reabrir com o processo aberto noutra unidade não observado | [contrato](../contracts/cli/processo-reabrir.md), [decisão](decisoes/processo-reabrir.md), [evidência](../contracts/evidencias/processo-reabrir.md) |
| `ies processo marcar NUP --marcador ID#NOME\|--retirar ID#NOME [--confirmar]` | **PARCIAL WEB** (#255), sem escrita | Leitura dos marcadores com unidade conferida; idempotência para atribuir já presente e retirar ausente; prévia para retirada presente. Lista parcial e identidade divergente recusadas. `--confirmar` de retirada falha fechado, sem POST; atribuir ausente exige catálogo da unidade ainda não observado. Requisições contadas pelo leitor de processo | Live NOT_RUN (0 req., nenhum resíduo); RED local NOT_RUN (Rust indisponível); replay pendente | [contrato](../contracts/cli/processo-marcar.md), [decisão](decisoes/processo-marcar.md), [live](validacao-live/2026-11-05-processo-marcar.md), [#255](https://github.com/franklinbaldo/ies/issues/255) |
| `ies processo criar --tipo TIPO --especificacao TEXTO [--interessado ...] [--nivel publico\|restrito] [--hipotese-legal ID#NOME] [--confirmar]` | **PENDENTE PR** (#245, PR #283) | Seleção de tipo Web 4.0/4.1, cadastro conferido, AJAX de interessado e hipótese legal com ID#NOME exatos para múltiplas opções; prévia, um POST criador e 3 GET de verificação (tela, árvore e propriedades: tipo/especificação/nível), sem retry | CI sintético e replay original: 15 cassettes/83 respostas (59 lidas, 3 recusadas, 21 não aplicáveis); **live NOT_RUN**, 0 pedidos, nenhum resíduo; ensaio autorizado só em treinamento | [contrato](../contracts/cli/processo-criar.md), [decisão](decisoes/processo-criar.md), [#245](https://github.com/franklinbaldo/ies/issues/245) |
| `ies documento DOC`, `salvar`, `buscar`, `assinar` | ver a [matriz de documento](matriz-derivacao-cli-documento.md) (4 caminhos) | Idem | Idem | Idem |
| `ies unidade listar-processos`, `colunas`, `recebidos`, `gerados`, `atual` | ver a [matriz da unidade](matriz-derivacao-cli-unidade.md) (6 caminhos) | Idem | Idem | Idem |
| `ies unidade listar [--atualizar]` | **DRAFT #244** | Leitura completa da lupa (4 GET + 1 POST/página), snapshot transacional SQLite, consulta guardada sem rede | Replay GO parcial; live UnB/GO NOT_RUN | [contrato](../contracts/cli/unidade-listar.md), [decisão](decisoes/unidade-listar.md) |
| `ies unidade pesquisar TERMO` | **DRAFT #244** | Pesquisa offline sem acento, sigla exata primeiro; coleta inicial pela lupa quando falta snapshot | Live NOT_RUN | [contrato](../contracts/cli/unidade-pesquisar.md), [decisão](decisoes/unidade-pesquisar.md) |
| `ies indice caminho` | **DRAFT #244** | Descobre o arquivo SQLite sem abrir banco ou enviar pedidos Web | Testes sintéticos no CI | [contrato](../contracts/cli/indice-caminho.md), [decisão](decisoes/indice-caminho.md) |
| `ies status` | **LOCAL**, diagnóstico Web limitado | Metadados/validação local de arquivos; GET opt-in do Controle | Não é login, certificação da sessão remota ou teste de permissões; evitar expandir flags antes das capacidades | [código](../src/status_cli.rs) |
| `ies setup` | **LOCAL** | Registra caminhos privados sem copiar credenciais | Não automatiza login, renovação ou troca de unidade; ampliar apenas se indispensável às operações já declaradas | [código](../src/configuracao.rs) |
| `ies login [--forcar]` | **PARCIAL WEB** (decisão do proprietário, 02/11/2026) | Login pela tela servida da raiz (SIP e módulo de Goiás), credenciais em arquivo `0600`, órgão exato, `hdnAcao` e botão observados, Painel pelo link do menu, gravação atômica por geração sob lock; sessão válida: 1 GET | Live PASS UnB, PEN1, PEN3, PEN4, UFRJ e GO; PEN2 bloqueada pelo ambiente (503, 403 do Cloudflare); mensagem real de credencial recusada. **Relogin único nas 9 leituras** (#200): só depois de `pagina_de_login_recebida_no_controle` ou `sessao_encerrada_rode_ies_login`, credenciais do mesmo arquivo, geração conferida sob o lock (outro processo já relogou: sem login), uma releitura do início, `relogin: true` e o login somado em `requisicoes`; escrita e prévia nunca relogam. Live PEN4 PASS com sessão desconhecida pelo servidor (9 pedidos) e geração trocada (2 pedidos); vencimento natural NOT_RUN; 302 do Controle para o login da instalação é sessão encerrada (live PEN4 PASS, 8 pedidos) ([live](validacao-live/2026-11-05-relogin.md)) | [contrato](../contracts/cli/sessao-login.md), [relogin](decisoes/sessao-login/relogin-de-leitura.md) |
| `ies esquema` | **PARCIAL LOCAL** | Lista nomes de comandos e versão | Entregar contrato/esquema útil se necessário, usando o comando existente; não criar outro inventário de comandos | [código](../src/esquema_cli.rs) |
| ~~`ies replay CENARIO`~~ | **RETIRADO** da CLI (05/11/2026) | O replay das cassettes são os testes `#[ignore]` do acervo, pelos leitores dos comandos acima; `scripts/replay` os lista e executa | Não volta à CLI; o binário de produção não liga `zip` nem `sha1` | [decisão](decisoes/acervo/replay-fora-da-cli.md) · [cassettes](cassettes.md#replay-do-acervo) |

## Histórico e transição para a base #206

As fatias #57, #58, #59, #60, #61, #88, #171 e as escritas de processo
não são mais stubs do roadmap inicial. A árvore, pesquisa, documento
interno/externo, lista de versões, unidade e escritas estão descritos na
tabela e nas matrizes específicas. Histórico das decisões anteriores
permanece rastreável no Git e nos relatórios de validação vinculados.
A PR #193 integrou o envio e a assinatura ao vivo; a #206 ampliou
esses ensaios, o relogin e a troca de unidade.

**Distinguir estado da main de validação live.** `PARCIAL WEB` pode
significar capacidade já escrita em instalações específicas, não
cobertura universal. `NOT_RUN` nunca vira `PASS` por CI ou replay.

**Critério de contagem:** **20 caminhos operacionais** na
`main@52be0e1`: 6 de unidade (incluindo `trocar`, #205), 4 de
documento, 5 de processo, mais `status`, `setup`, `login`,
`esquema` e `replay`. Excluem-se os agrupadores `ies processo`,
`ies documento` e `ies unidade` isolados, que exibem ajuda.
Flags e projeções não criam caminho independente. As 129 operações
históricas do Todos não correspondem a 129 comandos IES.

## Dependências após #193 e #206

```text
sessão Web autenticada -> Controle/índice -> árvore, listas e pesquisa
          |                      |                 |
          |                      |                 +-> documento atual/versões (#60/#88)
          |                      |                         +-> salvar editor (#89, observar)
          |                      +-> processo enviar/concluir/reabrir (escritas existentes)
          +-> relogin nas leituras (#200) / unidade trocar (#205)
documento assinar (#174) -> árvore + assinatura + verificação posterior
cópia/versão do documento (#204) -> ajuste de formato/semântica (#207)
```

O relogin é **uma única recuperação nas leituras** depois das recusas
tipadas de sessão; prévia e escrita não relogam. Trocar unidade é
**escrita do estado da sessão**, com PR e ensaio próprio, e
invalida os links assinados da unidade anterior. GET e POST
são classificados pelo efeito observado, não pelo verbo HTTP.

## Fila viva de implementação (referência: #56)

1. **#207** — concluir o desenho já aprovado de cópia HTML
   versionada/idempotente; `DOC.vN.txt` da #204 **será
   substituído pela #207**. Não expandir nomes ou contratos aqui.
2. **#89** — observar editor SEI 4/5 e, somente após evidência
   e decisão, implementar salvar com recusa anterior ao editor
   para assinado, confirmação e reconciliação.
3. **#60/#88** — fechar lacunas de documento e versão por
   instalação com evidência específica; `#61` cobre a pesquisa
   já funcional e aguarda eventuais filtros observados.
4. **#172/#173/#174** — ampliar ensaios live e casos de recusa
   de concluir/reabrir, enviar e assinar; **não** refazer o
   que já passou: enviar escrita **PASS UFRJ, UnB e PEN1**,
   assinar escrita **PASS PEN1 e UnB**.
5. **#200/#205** — relogin já presente nas leituras, com
   vencimento natural não observado; `unidade trocar` presente,
   **live PASS UFRJ**, outras instalações `NOT_RUN`.
6. **#183–#188/#201** — auditoria de infraestrutura e limites
   HTTP como trilha transversal, sem introduzir novos verbos.

A fila é prioridade por dependência, não prova de que uma issue
esteja concluída. Só evidência de critérios e relatório
`PASS/FAIL/NOT_RUN` permitem propor fechamento ao proprietário.
O CI valida os gates e testes offline, não substitui live.

## Gate manual para PRs de capacidade

Antes de abrir uma PR, preencher na descrição:
`comando existente` → `resultado atual` → `lacuna removida` →
`evidência de efeito` → `teste comportamental` →
`requests anteriores/posteriores` → `validação por instalação`.
Se só houver nova flag, parser ou contrato sem redução demonstrável
do caminho incompleto, redirecionar o esforço para as lacunas acima.
A nova capacidade **não** fecha issue ampla enquanto os critérios
de aceitação não forem integralmente comprovados.

## Handoff #253 — retornos programados (PR Draft)

| Comando | Estado deste branch | Evidência | Lacuna |
| --- | --- | --- | --- |
| `ies unidade retornos [--mes AAAA-MM] [--json]` | **PARCIAL WEB (PR DRAFT)**: GET do Controle, link publicado de `retorno_programado_listar`, calendário por mês e tabelas com legenda conferida; recusa paginação não observada | [Contrato](../contracts/cli/unidade-listar-retornos.md), [decisão](decisoes/unidade-listar-retornos.md), [observação](como-o-sei-funciona/retorno-programado.md), testes sintéticos Rust derivados de Todos @323eb01 | Replay original PASS 14/14; live em GO e PEN3 com `--mes 2026-11` PASS (18 requisições, 0 escrita). Navegação live sem `--mes` com calendário deslocado NOT_RUN; paginação independente ainda não observada. [Relatório](validacao-replay/2026-11-05-unidade-retornos.md). Não contar como capacidade em `main` antes da integração. |
