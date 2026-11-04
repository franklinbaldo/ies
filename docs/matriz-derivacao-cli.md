---
type: Guide
title: Matriz de derivação das capacidades da CLI IES
description: Cruzamento dos comandos declarados, código executável, evidências e lacunas reais
---

# Matriz de derivação da CLI — implementar antes de ampliar

**Base de derivação:** `main` auditada em 01/11/2026 após as PRs #137 e #138;
revalidar o SHA, o despacho e os checks a cada rodada. Propostas fora da `main`
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

## Matriz: 18 caminhos principais já expostos

| Comando existente | Estado na main | Já demonstra | Falta implementar, sem novo comando/flag | Referência |
| --- | --- | --- | --- | --- |
| `ies processo NUP` | **PARCIAL WEB** (#59 fatias 1 a 4) pela árvore; PARCIAL LOCAL/WEB em `--resumo`, `--historico-privado` e `--resultado-pesquisa-file` | Abre pelo caminho mais barato do índice (link da árvore 1, link colhido do Controle 2, entrada por id 3) ou pela pesquisa rápida/linha do Controle (3–4); documentos com pastas, movidos, anexados, pai do anexado, andamentos e propriedades (custos nos contratos); o Controle lido é colhido no índice | Live fatias 1-2 PASS UnB, GO, PEN1; propriedades e movidos NOT_RUN; marcadores | [contrato](../contracts/cli/processo-obter-arvore.md), [andamentos](../contracts/cli/processo-obter-andamentos.md), [pastas](../contracts/cli/processo-obter-pastas.md), [propriedades](../contracts/cli/processo-obter-propriedades.md), [#59](https://github.com/franklinbaldo/ies/issues/59) |
| `ies processo buscar TERMO [--pagina N]` | **PARCIAL WEB** (PENDENTE PR, Refs #58) | Pesquisa por protocolo pela sessão de `ies login`: link `protocolo_pesquisar` do Controle, GET da tela, POST derivado; 302 → `processo_unico` (só o id), lista com `total_declarado` e `faixa_observada` (separador de milhar), `sem_resultado` (saída 0); `--pagina N` por `navegar()`. Página 1: GET + GET + POST; página N: + 1 POST | Live: PASS UnB e GO, PEN1 sem índice, UFRJ/PEN2–4 NOT_RUN; filtros além do protocolo; número formatado após o 302 (abrir o processo é #59) | [contrato](../contracts/cli/processo-buscar.md), [#58](https://github.com/franklinbaldo/ies/issues/58) |
| `ies processo enviar NUP --para SIGLA` | **STUB** | Entrada/confirmar declarados | Prévia, autorização, destinatários, efeito e conferência pós-escrita, sem retry cego | [evidência](../contracts/evidencias/processo-enviar.md) |
| `ies processo concluir NUP` | **STUB** | Entrada/confirmar declarados | Prévia e execução controlada, verificação independente do resultado; efeito incerto tipado | [evidência](../contracts/evidencias/processo-concluir.md) |
| `ies documento DOC [DOC ...]` | **STUB** da obtenção Web geral; **PARCIAL LOCAL** com `--html-privado` | Identidade exata e texto capturado; exportação privada sem sobrescrever | Resolver DOC e link vigente pela árvore/pesquisa, GET autorizado com sessão vinculada, conteúdo e metadados, lote, histórico e `--versao` | [#60](https://github.com/franklinbaldo/ies/issues/60), [#88](https://github.com/franklinbaldo/ies/issues/88) |
| `ies documento salvar DOC` | **PARCIAL LOCAL**; `--confirmar` remoto bloqueado | Prévia, diff/cópias de trabalho locais e inventário versionado | Detectar versão/assinatura/conflitos no SEI; salvar com efeito e reconciliação; preservar rascunhos | [#89](https://github.com/franklinbaldo/ies/issues/89) |
| `ies documento buscar [--assunto]` | **STUB** | Gramática/entrada Clap; pesquisa textual apenas em `replay` | Busca Web por critérios observados, identidade, paginação e erros de acesso, sem promover replay a busca live | [#61](https://github.com/franklinbaldo/ies/issues/61) |
| `ies documento assinar DOC` | **STUB** | Gramática/confirmar declarados | Prévia, permissões, assinatura autorizada e confirmação independente dos efeitos | [evidência](../contracts/evidencias/documento-assinar.md) |
| `ies unidade listar-processos [--pagina N]` | **PARCIAL WEB** (PENDENTE PR, decisão do proprietário, 02/11/2026) | Visão D com colunas, categoria observada na R do mesmo GET e `colunas_ausentes`; troca R→D sem volta ou D direta, sem restaurar visão nem página (a sessão fica na D); 1 GET + 0 a 2 POST, 1 GET em regime estável | Ensaio live da troca sem restauração por instalação; efeito `PreferenciaDeVisualizacao`; colunas de ícone na PEN1 | [contrato](../contracts/cli/unidade-listar-processos-visao-detalhada.md) |
| `ies unidade colunas [--definir todas\|LISTA [--confirmar]]` | **PARCIAL WEB** (`--definir`: PENDENTE PR) | 12 opções do catálogo do Nível de detalhe com `disponivel` (a UnB 4.0.9 publica 11) e `marcada`; D: 2 GET; R: 2 GET + 1 POST D, e a sessão fica na D. `--definir`: prévia sem salvar; `--confirmar`: 1 POST de salvar + 1 GET de verificação, reconciliação e saída 3 | Ensaio live da leitura e da escrita | [leitura](../contracts/cli/unidade-colunas.md), [escrita](../contracts/cli/unidade-definir-colunas.md) |
| `ies unidade recebidos [--pagina N]` | **PARCIAL WEB** (decisão do proprietário, 02/11/2026) | Lista Recebidos: página observada com total e faixa; `--pagina N` com 1 GET + até 1 POST; sessão na D: troca para R e fica nela (+1 POST) | Ensaio live da nova sintaxe; efeitos do POST por instalação | [contrato](../contracts/cli/unidade-recebidos-gerados.md) |
| `ies unidade gerados [--pagina N]` | **PARCIAL WEB** (idem) | Lista Gerados, mesma regra e mesmo transporte | Idem | [contrato](../contracts/cli/unidade-recebidos-gerados.md) |
| `ies unidade atual` | **PARCIAL WEB** | Rótulo da unidade na página de Controle identificável | Validar em instalações autorizadas e garantir contexto da sessão; não infere permissão nem troca unidade | [contrato](../contracts/cli/unidade-atual.md) |
| `ies status` | **LOCAL**, diagnóstico Web limitado | Metadados/validação local de arquivos; GET opt-in do Controle | Não é login, certificação da sessão remota ou teste de permissões; evitar expandir flags antes das capacidades | [código](../src/status_cli.rs) |
| `ies setup` | **LOCAL** | Registra caminhos privados sem copiar credenciais | Não automatiza login, renovação ou troca de unidade; ampliar apenas se indispensável às operações já declaradas | [código](../src/configuracao.rs) |
| `ies login [--forcar]` | **PARCIAL WEB** (PENDENTE PR, decisão do proprietário, 02/11/2026) | Login pela tela servida da raiz (SIP e módulo de Goiás), credenciais em arquivo `0600`, órgão exato, `hdnAcao` e botão observados, Painel pelo link do menu, gravação atômica por geração sob lock; sessão válida: 1 GET | Ensaio live por instalação (UnB, PEN1, UFRJ, GO); mensagem real de credencial recusada; relogin nos outros comandos só depois de falha tipada de sessão vencida (futuro) | [contrato](../contracts/cli/sessao-login.md) |
| `ies esquema` | **PARCIAL LOCAL** | Lista nomes de comandos e versão | Entregar contrato/esquema útil se necessário, usando o comando existente; não criar outro inventário de comandos | [código](../src/esquema_cli.rs) |
| `ies replay CENARIO` | **LOCAL** no acervo suportado | Reproduz capturas/cassettes verificadas offline: caixa, pesquisa, documento e histórico | É oráculo de testes, não substitui implementação Web dos comandos acima | [guia](replay.md) |

**Atualização #137:** a visão detalhada D do GET existente pode produzir
lista completa apenas com cobertura declarada demonstrada; continua um GET,
zero POST e **sem ensaio live autorizado**. Persistem 11 caminhos
incompletos e 3 locais; nenhum virou capacidade Web integral. A #57 segue
aberta para a paginação ativa.

**Atualização #138 (integrada após revisão):** `--pagina-arquivo` recusa
complementos locais ausentes, inseguros ou acima do limite antes de acessar
a sessão ou executar GET potencialmente administrativo. O teto agregado
de 48 MB é conferido novamente após o GET, reutilizando os bytes já lidos.
Para complemento inviável, elimina um GET evitável (até 1 → 0); com
complementos válidos mantém até 1 GET e 0 POST. Os gates Rust/OKF/20 KB e
GitGuardian passaram no HEAD revisado. Não habilita paginação ativa, não
promove `PARCIAL WEB` e não equivale a validação live.

**Atualização #141 (integrada):** existe, só offline, o primitivo que deriva
o POST de um formulário servido ([decisão 0002](arquitetura/0002-submeter-formulario-observado.md)),
com envio injetável e sem cliente HTTP real. Nenhum comando o usa:
`processo buscar` segue **STUB** e `--resultado-pesquisa-file`, **PARCIAL
LOCAL**. Validação live: **NOT_RUN**.

**Atualização #58 (integrada em #142/#143):** a pesquisa de protocolo
interpreta o 302 da busca exata, com o `Location` relativo para
`procedimento_trabalhar` gravado nas cassettes de UnB, PEN1, UFRJ e
Goiás, como `IdProcedimento`, **sem segui-lo**. Também aplica as regras
do `onsubmit` lidas do JavaScript servido. O ensaio offline passou nos
8 redirecionamentos e nos 16 formulários reais. Ainda sem CLI e sem
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
intermitente (2 de 8 GETs). PEN2 ficou NOT_RUN (503). `unidade atual` passa a ter validação live; os estados da matriz
não mudam, porque a paginação ativa (#57) continua ausente.

**PENDENTE PR #57 (página observada e `--pagina N`):** `unidade
listar-processos` deixa de recusar caixa paginada: devolve a página que a
tela mostra com `total_declarado` e `faixa_observada` por tabela, zero POST;
`unidade recebidos|gerados --pagina N` (antes `--categoria T --pagina N`)
faz no máximo um POST do paginador da tabela,
com efeito `PaginaOuFiltroLembrado` e sem restauração. Ensaio live
**NOT_RUN**. Inventário das outras leituras paginadas, ainda fora da
[regra geral](../contracts/cli.md): `processo NUP --resumo` e
`replay --numero-processo` recusam página parcial
(`paginas_do_controle_nao_obtidas`) porque ausência numa página não prova
ausência na caixa; `--resultado-pesquisa-file` e `--historico-privado` não
leem total nem faixa da resposta; a lista de versões de documento só
aceita legenda igual às linhas. Alinhar cada uma exige evidência da
legenda e, para `--pagina N`, do paginador daquela tela.

**Decisão do proprietário (02/11/2026), exceção explícita à regra de
priorização:** Recebidos e Gerados são listas independentes do SEI
(paginador, total, faixa e página lembrada próprios). Por isso ganham
os subcomandos `ies unidade recebidos` e `ies unidade gerados`, os únicos
com `--pagina N`; `listar-processos` devolve o array das duas listas e
perde `--categoria` e `--pagina`, sem alias. Não é capacidade nova da
operação `unidade.listar_processos`: mesmo GET, mesmo POST e mesmas
recusas ([decisão](decisoes/unidade-listar-processos.md)). A exceção não
autoriza outros verbos.

**PENDENTE PR (visão detalhada por padrão, decisão do proprietário,
02/11/2026):** `listar-processos` lê a visão D (uma lista com colunas e a
categoria observada por processo), troca de visão sem restaurar (a sessão
fica na D; decisão do proprietário, 02/11/2026: restaurar desperdiçava
requisições), e volta a ter `--pagina N` (página da D, que fica lembrada); `--pagina-arquivo` sai. `unidade
colunas` lê o Nível de detalhe sem salvar. A
[decisão](decisoes/unidade-listar-processos.md) registra a exceção à regra
de não criar verbos para `colunas`, que integra a mesma operação.
Ensaio live **NOT_RUN**.

**PENDENTE PR (escrita das colunas, decisão do proprietário "B",
02/11/2026):** `unidade colunas --definir todas|LISTA [--confirmar]` é a
operação `unidade.definir_colunas`, a primeira **escrita** remota do IES: a
opção pertence ao comando já exposto (não é verbo novo) e substitui a
alternativa de salvar e restaurar a cada listagem. Sem `--confirmar`, prévia
com os campos do POST e zero escrita; com ele, um POST de salvar e um GET de
verificação, reconciliação por leitura e saída 3 para efeito incerto; vinda
da R, a sessão fica na D. `listar-processos` passa a sugerir
`--definir todas` para as `colunas_ausentes`; `ies setup` não aplica nada.
Offline com telas sintéticas de 12 e 11 opções; ensaio live **NOT_RUN**
([decisão](decisoes/unidade-definir-colunas.md),
[roteiro](validacao-live/roteiro-definir-colunas.md)).

**PENDENTE PR (login nativo, decisão do proprietário, 02/11/2026):** `ies
login [--forcar]` é a operação `sessao.login`, um comando novo por decisão
do proprietário: até aqui a sessão vinha do Todos ou do navegador, copiada
à mão. Lê a origem, confere a sessão gravada com 1 GET (válida: sem login),
loga pela raiz (2 ou 3 GETs, 1 POST e os saltos até o Controle) e grava
cookie e URL de forma atômica, com marcador de geração e lock. Os outros
comandos continuam sem login automático. Offline com telas sintéticas das
quatro famílias; ensaio live **NOT_RUN**
([decisão](decisoes/sessao-login.md), [roteiro](validacao-live/roteiro-login.md)).

**PENDENTE PR #58 (pesquisa Web de processo):** `ies processo buscar TERMO
[--pagina N]` substitui o stub `[--assunto]` (mesmo verbo). Segue só o link
`protocolo_pesquisar` que o Controle publicou, deriva o POST da tela servida
e interpreta o 302 sem segui-lo. Saídas `processo_unico`, `lista` e
`sem_resultado`, esta última uma resposta observada, não inexistência.
Offline com telas sintéticas modeladas na UnB 4.0.9 (03/11/2026); ensaio
live do comando IES: PASS na UnB e em GO ([relatório](validacao-live/2026-11-03-processo-buscar.md)).

**Critério de contagem:** 18 caminhos operacionais (o 18º é `ies login`, PENDENTE PR), desconsiderando
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
    │   └── unidade listar-processos → paginação/efeitos (#57)
    ├── formulário de pesquisa observado → processo buscar (#58)
    │   └── resolver NUP/ID → processo NUP: árvore/histórico/propriedades (#59)
    │       └── links reais de documento → documento DOC (#60)
    │           └── versão atual e --versao N (#88)
    │               └── documento salvar: prévia → escrita/verificação (#89)
    └── pesquisa de documentos observada → documento buscar (#61)
Pós-leituras e autorização específica: processo enviar/concluir; documento assinar.
```

A cadeia representa **dependências técnicas a demonstrar**, não afirma
que cada GET/POST seja autorizado, livre de efeito ou disponível em todas as
instalações. Login automático não está comprovado. A
[PR #131](https://github.com/franklinbaldo/ies/pull/131) propõe GET
opt-in de documento, mas **não está na main** e depende de revisão
independente de segurança; não contabilizar como `documento DOC` resolvido.

## Fila de implementação, sem ampliação da CLI

1. **P0 #57** — fazer `unidade listar-processos` cobrir paginação real
   pela Web quando autorizada, ou falhar fechado; medir GET/POST/efeitos.
2. **P0 #58** — implementar pesquisa Web no caminho `processo buscar`
   já existente e viabilizar resolução de `processo NUP` fora da caixa.
3. **P0 #59** — implementar `processo NUP` sem `--resumo`: árvore,
   documentos, histórico e propriedades reais, com projeções existentes.
4. **P0 #60 e #88** — implementar `documento DOC` e `--versao N` pelo
   histórico observado e links frescos, sem presumir editor seguro.
5. **P0 #61** — implementar `documento buscar` por critérios existentes.
6. **P1 #89 e escritas já declaradas** — entregar `documento salvar`,
   `processo enviar/concluir` e `documento assinar` com prévia, efeito
   seguro e reconciliação; nunca testar escrita incerta em produção.

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
