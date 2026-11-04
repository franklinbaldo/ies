---
type: Guide
title: Matriz de derivação das capacidades da CLI IES
description: Cruzamento dos comandos declarados, código executável, evidências e lacunas reais
---

# Matriz de derivação da CLI — implementar antes de ampliar

**Base de derivação:** `main` auditada em 04/11/2026 após a PR #171
(`62fb385`); revalidar o SHA, o despacho e os checks a cada rodada. Propostas fora da `main`
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

## Matriz: 19 caminhos principais já expostos

| Comando existente | Estado na main | Já demonstra | Falta implementar, sem novo comando/flag | Referência |
| --- | --- | --- | --- | --- |
| `ies processo NUP` | **PARCIAL WEB** (#59 cumprida) pela árvore; `--resumo` e as opções de captura privada aposentados em 04/11/2026 (decisão do proprietário) | Abre pelo caminho mais barato do índice (link da árvore 1, link colhido do Controle 2, entrada por id 3) ou pela pesquisa rápida/linha do Controle (3–4); documentos com pastas, movidos (também de volta), ícone e cancelamento, anexados, pai do anexado, onde está aberto ou sobrestado (0 req.), andamentos, propriedades e marcadores em lista (custos nos contratos); o Controle lido é colhido no índice | Live PASS UnB, GO, PEN1 (propriedades e movidos: UnB/PEN1; marcadores: UnB); [replay do acervo](validacao-replay/2026-11-04-processo-obter-acervo.md) | [contrato](../contracts/cli/processo-obter-arvore.md), [andamentos](../contracts/cli/processo-obter-andamentos.md), [pastas](../contracts/cli/processo-obter-pastas.md), [movidos](../contracts/cli/processo-obter-movidos.md), [propriedades](../contracts/cli/processo-obter-propriedades.md), [marcadores](../contracts/cli/processo-obter-marcadores.md), [situação](../contracts/cli/processo-obter-situacao.md), [#59](https://github.com/franklinbaldo/ies/issues/59) |
| `ies processo buscar TERMO [--pagina N]` | **PARCIAL WEB** (#58, núcleo em main) | Pesquisa por protocolo pela sessão de `ies login`: link `protocolo_pesquisar` do Controle, GET da tela, POST derivado; 302 → `processo_unico` (só o id), lista com `total_declarado` e `faixa_observada` (separador de milhar), `sem_resultado` (saída 0); `--pagina N` por `navegar()`; linha com acerto em documento (GO) publica só o processo; 403 da borda é `bloqueio_de_seguranca_da_instalacao`. Página 1: GET + GET + POST; página N: + 1 POST. Replay do acervo: 52 cassettes nas 7 instalações | Live: PASS UnB, GO e PEN3 (NUP fora da caixa localizado), PEN1 sem índice, UFRJ/PEN2/PEN4 NOT_RUN; linhas a menos que a faixa escrita na barra (GO 5.0.3, 9 de 10) saem com `linhas_ausentes_da_faixa`, sem afirmar completude; termo só de dígitos ao vivo PASS na PEN3 (o 403 do Cloudflare veio no GET do Controle, não pelo termo); filtros além do protocolo | [contrato](../contracts/cli/processo-buscar.md), [replay](validacao-replay/2026-11-04-processo-buscar-acervo.md), [live PEN3](validacao-live/2026-11-04-processo-buscar.md), [#58](https://github.com/franklinbaldo/ies/issues/58) |
| `ies processo enviar NUP --para SIGLA` | **STUB** | Entrada/confirmar declarados | Prévia, autorização, destinatários, efeito e conferência pós-escrita, sem retry cego | [evidência](../contracts/evidencias/processo-enviar.md) |
| `ies processo concluir NUP [--confirmar]` | **PARCIAL WEB** (#172) | Prévia só com a abertura (2–3 req.); uma escrita pela forma da barra: tela "Conclusão de Processo" + POST (4.1/5.0) ou GET de `concluirProcesso()` (4.0); verificação por `aberto_em` numa leitura nova; efeito incerto sai com 3, sem retry; a frio 5 GET + 1 POST / 6 GET | Live PASS UFRJ 4.1.5 (`tela`) e UnB 4.0.9 (`funcao`), requisições iguais ao orçamento; [replay](validacao-replay/2026-11-04-processo-concluir-acervo.md) 219 escritas verificadas; PEN1 5.0.4 não observada; concluir com reabertura programada, vários processos | [contrato](../contracts/cli/processo-concluir.md), [decisão](decisoes/processo-concluir.md), [live](validacao-live/2026-11-04-processo-concluir.md) |
| `ies processo reabrir NUP [--confirmar]` | **PARCIAL WEB** (#172) | Prévia só com a abertura; `reabrirProcesso()` da barra → GET da tela do nó → URL `procedimento_reabrir` (corpo da função no 4.0, `var linkReabrirProcesso` no 4.1/5.0) → um GET (a escrita); verificação pela unidade da sessão em `aberto_em`; efeito incerto 3, sem retry; a frio 6 GET | Live PASS UFRJ 4.1.5 (`variavel`) e UnB 4.0.9 (`funcao`), 5 req. cada; [replay](validacao-replay/2026-11-04-processo-reabrir-acervo.md) 11 reaberturas verificadas; reabrir com o processo aberto noutra unidade não observado | [contrato](../contracts/cli/processo-reabrir.md), [decisão](decisoes/processo-reabrir.md), [evidência](../contracts/evidencias/processo-reabrir.md) |
| `ies documento DOC [DOC ...]` | **PARCIAL WEB** (#60 fatias 1 a 3, #88), internos e externos; **PARCIAL LOCAL** com `--html-privado` | Pesquisa rápida do número SEI (4 req. com o `action` guardado, 5 pelo Controle) → tela → árvore conferida → GET do `Nos[i].src` assinado (`documento_visualizar`); texto (entidades decodificadas) e metadados (rótulo, tipo, ícone, processo, unidade); lote em sequência; `--saida` grava `DOC.txt` 0600; fatia 2: índice número SEI → processo/documento aprendido de toda árvore, caminho link da árvore (2) → link de trabalho (3) → entrada da lista branca com `id_documento` (4, aceite por instalação) → pesquisa rápida, palpite desmentido esquecido com uma queda, `--processo NUP`, pasta fechada aberta só se o documento não está à vista; fatia 3: externos pelo `.src` `documento_download_anexo` assinado (GET binário, PDF/TXT/HTML recebido, `IES_LIMITE_ANEXO_BYTES` 64 MiB, SHA-256, `--saida` → `DOC.pdf|txt|html` 0600 sem sobrescrever); `--versao N` pelo link assinado `documento_versao_listar` da barra do DOCUMENTO (G+2, conteúdo atual não pedido); `versao_atual` do rodapé servido "versão N" (0 GET a mais) ou da lista; cópia `DOC.vN.html` + base + manifesto com `--saida` só com `editarConteudo('N')`, N atual e sem `infra_hash`; `--sem-copia`; recusa tipada de sem acesso, cancelado, número diferente e inexistente | PDF e TXT ao vivo (só HTML recebido na PEN1); outros tipos só depois de observados; parágrafos no texto; versões PASS PEN4 5.0.4 ([ensaio](validacao-live/2026-11-04-documento-versoes.md)); versões em UnB/PEN1 NOT_RUN. Live PASS UnB ([ensaio](validacao-live/2026-11-04-documento-obter.md), [replay](validacao-replay/2026-11-04-documento-obter-acervo.md)) | [contrato](../contracts/cli/documento-obter-web.md), [versões](../contracts/cli/documento-obter-versoes.md), [decisão](decisoes/documento-obter.md), [#60](https://github.com/franklinbaldo/ies/issues/60), [#88](https://github.com/franklinbaldo/ies/issues/88) |
| `ies documento salvar DOC` | **PARCIAL LOCAL**; `--confirmar` remoto bloqueado | Prévia, diff/cópias de trabalho locais e inventário versionado | Detectar versão/assinatura/conflitos no SEI; salvar com efeito e reconciliação; preservar rascunhos | [#89](https://github.com/franklinbaldo/ies/issues/89) |
| `ies documento buscar TERMO [--pagina N]` | **PARCIAL WEB** (#61) | Mesma Pesquisa de `processo buscar` em documentos (rádio D e caixas de tipo): GET+GET+POST, página N +1 POST; 302 com `id_documento` → `documento_unico` (só ids, "um acerto"); lista com `total_declarado`/`faixa_observada` e número SEI, rótulo, NUP, unidade, inclusão e anexo; `sem_resultado` saída 0. Replay do acervo: 28 respostas em documentos lidas (4 de GO com `linhas_ausentes_da_faixa`), 14/14 páginas conferidas | Live PASS PEN3 5.0.4 ([ensaio](validacao-live/2026-11-04-documento-buscar.md), [replay](validacao-replay/2026-11-04-documento-buscar-acervo.md)); número SEI exato só de dígitos ao vivo PASS na PEN3 (302 → `documento_unico`, zeros à esquerda opcionais); o bloqueio do Cloudflare não depende do termo; UnB e GO ao vivo; filtros (assunto, tipo, período) só com uso | [contrato](../contracts/cli/documento-buscar.md), [#61](https://github.com/franklinbaldo/ies/issues/61) |
| `ies documento assinar DOC` | **STUB** | Gramática/confirmar declarados | Prévia, permissões, assinatura autorizada e confirmação independente dos efeitos | [evidência](../contracts/evidencias/documento-assinar.md) |
| `ies unidade listar-processos`, `colunas`, `recebidos`, `gerados`, `atual` | ver a [matriz da unidade](matriz-derivacao-cli-unidade.md) (5 caminhos) | Idem | Idem | Idem |
| `ies status` | **LOCAL**, diagnóstico Web limitado | Metadados/validação local de arquivos; GET opt-in do Controle | Não é login, certificação da sessão remota ou teste de permissões; evitar expandir flags antes das capacidades | [código](../src/status_cli.rs) |
| `ies setup` | **LOCAL** | Registra caminhos privados sem copiar credenciais | Não automatiza login, renovação ou troca de unidade; ampliar apenas se indispensável às operações já declaradas | [código](../src/configuracao.rs) |
| `ies login [--forcar]` | **PARCIAL WEB** (decisão do proprietário, 02/11/2026) | Login pela tela servida da raiz (SIP e módulo de Goiás), credenciais em arquivo `0600`, órgão exato, `hdnAcao` e botão observados, Painel pelo link do menu, gravação atômica por geração sob lock; sessão válida: 1 GET | Live PASS UnB, PEN1, PEN3, PEN4, UFRJ e GO; PEN2 bloqueada pelo ambiente (503, 403 do Cloudflare); mensagem real de credencial recusada; relogin nos outros comandos só depois de falha tipada de sessão vencida (futuro) | [contrato](../contracts/cli/sessao-login.md) |
| `ies esquema` | **PARCIAL LOCAL** | Lista nomes de comandos e versão | Entregar contrato/esquema útil se necessário, usando o comando existente; não criar outro inventário de comandos | [código](../src/esquema_cli.rs) |
| `ies replay CENARIO` | **LOCAL** no acervo suportado | Reproduz capturas/cassettes verificadas offline: caixa, pesquisa, documento e histórico | É oráculo de testes, não substitui implementação Web dos comandos acima | [guia](replay.md) |

**Atualização #137:** a visão detalhada D do GET existente pode produzir
lista completa apenas com cobertura declarada demonstrada; continua um GET,
zero POST e **sem ensaio live autorizado**. (Superada: a página observada
e `--pagina N` de #57 estão em main.)

**Atualização #138 (integrada após revisão):** `--pagina-arquivo` recusa
complementos locais ausentes, inseguros ou acima do limite antes de acessar
a sessão ou executar GET potencialmente administrativo. O teto agregado
de 48 MB é conferido novamente após o GET, reutilizando os bytes já lidos.
Para complemento inviável, elimina um GET evitável (até 1 → 0); com
complementos válidos mantém até 1 GET e 0 POST. (Superada: a opção saiu de
`listar-processos` em 02/11/2026 e de `processo NUP --resumo` em
04/11/2026.)

**Atualização #141 (integrada):** existe, só offline, o primitivo que deriva
o POST de um formulário servido ([decisão 0002](arquitetura/0002-submeter-formulario-observado.md)),
com envio injetável e sem cliente HTTP real. Nenhum comando o usa:
`processo buscar` seguia **STUB**. (Superada: `processo buscar` está em main
pela #58; `--resultado-pesquisa-file` foi aposentado em 04/11/2026.)

**Atualização #58 (integrada em #142/#143):** a pesquisa de protocolo
interpreta o 302 da busca exata, com o `Location` relativo para
`procedimento_trabalhar` gravado nas cassettes de UnB, PEN1, UFRJ e
Goiás, como `IdProcedimento`, **sem segui-lo**. Também aplica as regras
do `onsubmit` lidas do JavaScript servido. O ensaio offline passou nos
8 redirecionamentos e nos 16 formulários reais. Sem CLI naquela fatia e sem
mudança de estado; o orçamento previsto é 1 GET da tela e 1 POST, com 0
GET ao destino. Lacunas: corpo real do POST (via
[coleta do formato](coleta-formato-post.md)), página "sem resultados"
não observada e controles que o navegador enviaria e a derivação omite.

**Ensaio live de 02/11/2026** ([relatório](validacao-live/2026-11-02-leitura-controle.md)):
primeira validação autenticada, com login pelo Todos e 3 GETs do IES por
instalação, sem POST. `status --verificar-controle-web` e `unidade atual`
deram PASS em UnB, UFRJ, PEN1, PEN3 e PEN4. `unidade listar-processos`
deu PASS em UnB e UFRJ, e recusa correta de caixa paginada em PEN1, PEN3
e PEN4. GO falhou pelo link sigiloso não reconhecido; corrigido, passou
com recusa correta da caixa paginada, mas com `resposta_inesperada`
intermitente (2 de 8 GETs). PEN2 ficou NOT_RUN (503). `unidade atual` passa a ter validação live; a paginação ativa
(#57) veio depois e passou ao vivo em 04/11/2026.

**#57 (caixa da unidade, feita):** página observada e `--pagina N`, listas
independentes, visão detalhada por padrão e escrita das colunas estão na
[matriz da unidade](matriz-derivacao-cli-unidade.md).

**Login nativo (decisão do proprietário, 02/11/2026; em main):** `ies
login [--forcar]` é a operação `sessao.login`, um comando novo por decisão
do proprietário: até aqui a sessão vinha do Todos ou do navegador, copiada
à mão. Lê a origem, confere a sessão gravada com 1 GET (válida: sem login),
loga pela raiz (2 ou 3 GETs, 1 POST e os saltos até o Controle) e grava
cookie e URL de forma atômica, com marcador de geração e lock. Os outros
comandos continuam sem login automático. Live PASS em UnB, PEN1, PEN3,
PEN4, UFRJ e GO ([colheita](validacao-live/2026-11-03-colheita-do-controle.md),
[paginação](validacao-live/2026-11-04-unidade-paginacao.md)); PEN2 recusada
pelo ambiente ([decisão](decisoes/sessao-login.md), [roteiro](validacao-live/roteiro-login.md)).

**#58 (pesquisa Web de processo; núcleo em main):** `ies processo buscar TERMO
[--pagina N]` substitui o stub `[--assunto]` (mesmo verbo). Segue só o link
`protocolo_pesquisar` que o Controle publicou, deriva o POST da tela servida
e interpreta o 302 sem segui-lo. Saídas `processo_unico`, `lista` e
`sem_resultado`, esta última uma resposta observada, não inexistência.
Offline com telas sintéticas modeladas na UnB 4.0.9 (03/11/2026); ensaio
live do comando IES: PASS na UnB e em GO ([relatório](validacao-live/2026-11-03-processo-buscar.md)).

**Critério de contagem:** 18 caminhos operacionais (o 18º é `ies login`), desconsiderando
`ies processo`, `ies documento` e `ies unidade` isolados, que mostram
ajuda. Flags e projeções pertencem ao comando existente e não são novas
capacidades independentes. As 129 operações do inventário do Todos são
**evidências históricas**, não 129 comandos IES implementados, aprovados
ou prioritários. Esta contagem descreve a superfície de `main`, não
testes live aprovados.

## Dependências: derivar entregas a partir da base existente

```text
setup/status local + origem HTTPS + cookie/URL da mesma sessão
    ├── Controle Web (GET existente)
    │   ├── unidade atual (já parcialmente atendida)
    │   └── unidade listar-processos → paginação/efeitos (#57, feito)
    ├── formulário de pesquisa observado → processo buscar (#58, núcleo feito)
    │   └── resolver NUP/ID → processo NUP: árvore/histórico/propriedades/marcadores (#59, feito)
    │       └── links reais de documento → documento DOC (#60)
    │           └── versão atual e --versao N (#88)
    │               └── documento salvar: prévia → escrita/verificação (#89)
    └── pesquisa de documentos observada → documento buscar (#61)
Pós-leituras e autorização específica: processo enviar/concluir; documento assinar.
```

A cadeia representa **dependências técnicas a demonstrar**, não afirma
que cada GET/POST seja autorizado, livre de efeito ou disponível em todas as
instalações. Login automático nos outros comandos não existe. A
[PR #131](https://github.com/franklinbaldo/ies/pull/131) (draft) propõe GET
opt-in de documento por arquivo de link privado, **não está na main** e
parte de uma arquitetura substituída (índice e links da árvore); não
contabilizar como `documento DOC` resolvido.

## Fila de implementação, sem ampliação da CLI

Feitas: **#57** (página observada e `--pagina N` da caixa, listas
independentes, colunas; [matriz da unidade](matriz-derivacao-cli-unidade.md)),
**#59** (`processo NUP` pela árvore com índice local) e o núcleo de
**#58** (`processo buscar`). A fila segue daí, uma fatia por PR:

1. **#60 fatia 1** — `documento DOC --processo NUP` pelo link da árvore,
   com o índice de DOC; reaproveita a abertura de #59.
2. **#60 fatia 2** — `documento DOC` sem `--processo`, lote do mesmo
   processo e documento externo.
3. **#88** — versão atual e `--versao N` pelo link publicado de versões.
4. **#61** — `documento buscar` pela pesquisa já validada em #58.
5. **Escritas já declaradas** — `processo concluir` e `processo enviar`
   (verificação pela árvore que #59 lê), depois #89 (`documento salvar`)
   e `documento assinar`, com prévia, efeito seguro e reconciliação;
   nunca testar escrita incerta em produção.

Em paralelo, sem competir com a fila: fatia final de #58 e varredura live
de `processo NUP` e `processo buscar` em UFRJ, PEN3 e PEN4.

As prioridades podem mudar conforme dependências e autorização. Não
abrir novos comandos como substituto de completar estes. Para cada
capacidade, exigir **CONTRACT → RED observado → GREEN → VALIDATED**;
se a fase ou o ensaio externo não ocorrer, registrar `NOT_RUN` e
o motivo. PASS de CI ou replay offline não equivale a validação Web
autenticada em UnB, UFRJ, Goiás ou PEN.

## Gate manual para PRs de capacidade

Antes de abrir uma PR, preencher na descrição:
`comando existente` → `resultado atual` → `lacuna removida` →
`evidência de efeito` → `teste comportamental` →
`requests anteriores/posteriores` → `validação por instalação`.
Se só houver nova flag, parser ou contrato sem redução demonstrável
do caminho incompleto, redirecionar o esforço para as lacunas acima.
A nova capacidade **não** fecha issue ampla enquanto os critérios
de aceitação não forem integralmente comprovados.
