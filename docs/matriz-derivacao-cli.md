---
type: Guide
title: Matriz de derivação das capacidades da CLI IES
description: Cruzamento dos comandos declarados, código executável, evidências e lacunas reais
---

# Matriz de derivação da CLI — implementar antes de ampliar

**Base de derivação:** `main` em `1d38ef84cf83f7d3754eca34232955996703c64d`,
01/11/2026, após integração da PR #137. Revalidar a main e seus checks
a cada rodada; propostas ainda não mescladas não são capacidades aprovadas. Fontes: [superfície Clap e despacho](../src/main.rs),
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

## Matriz: 14 caminhos principais já expostos

| Comando existente | Estado na main | Já demonstra | Falta implementar, sem novo comando/flag | Referência |
| --- | --- | --- | --- | --- |
| `ies processo NUP` | **STUB** da leitura integral; PARCIAL LOCAL/WEB em `--resumo`, `--historico-privado` e `--resultado-pesquisa-file` | NUP tipado; resumo da caixa por GET existente ou arquivo, busca e histórico em HTML privado | Resolver NUP fora da caixa, obter árvore/documentos, histórico e propriedades pela sessão autorizada; verificar identidade e efeitos | [#58](https://github.com/franklinbaldo/ies/issues/58), [#59](https://github.com/franklinbaldo/ies/issues/59) |
| `ies processo buscar [--assunto]` | **STUB** | Gramática/entrada Clap | Executar formulário de pesquisa Web observado, paginação, filtros e resultados seguros; aproveitar resolução para `processo NUP` | [#58](https://github.com/franklinbaldo/ies/issues/58) |
| `ies processo enviar NUP --para SIGLA` | **STUB** | Entrada/confirmar declarados | Prévia, autorização, destinatários, efeito e conferência pós-escrita, sem retry cego | [evidência](../contracts/evidencias/processo-enviar.md) |
| `ies processo concluir NUP` | **STUB** | Entrada/confirmar declarados | Prévia e execução controlada, verificação independente do resultado; efeito incerto tipado | [evidência](../contracts/evidencias/processo-concluir.md) |
| `ies documento DOC [DOC ...]` | **STUB** da obtenção Web geral; **PARCIAL LOCAL** com `--html-privado` | Identidade exata e texto capturado; exportação privada sem sobrescrever | Resolver DOC e link vigente pela árvore/pesquisa, GET autorizado com sessão vinculada, conteúdo e metadados, lote, histórico e `--versao` | [#60](https://github.com/franklinbaldo/ies/issues/60), [#88](https://github.com/franklinbaldo/ies/issues/88) |
| `ies documento salvar DOC` | **PARCIAL LOCAL**; `--confirmar` remoto bloqueado | Prévia, diff/cópias de trabalho locais e inventário versionado | Detectar versão/assinatura/conflitos no SEI; salvar com efeito e reconciliação; preservar rascunhos | [#89](https://github.com/franklinbaldo/ies/issues/89) |
| `ies documento buscar [--assunto]` | **STUB** | Gramática/entrada Clap; pesquisa textual apenas em `replay` | Busca Web por critérios observados, identidade, paginação e erros de acesso, sem promover replay a busca live | [#61](https://github.com/franklinbaldo/ies/issues/61) |
| `ies documento assinar DOC` | **STUB** | Gramática/confirmar declarados | Prévia, permissões, assinatura autorizada e confirmação independente dos efeitos | [evidência](../contracts/evidencias/documento-assinar.md) |
| `ies unidade listar-processos` | **PARCIAL WEB** | GET único da tela resumida **ou detalhada já exibida** (`D`), com identidade, total, faixa e exportação verificados; páginas privadas opcionais; recusa paginação sem intervalo | Paginação Web **ativa**, cobertura de recebidos/gerados, controle de contexto e efeitos do POST; visão D não inventa categorias ou snapshot | [#57](https://github.com/franklinbaldo/ies/issues/57) |
| `ies unidade atual` | **PARCIAL WEB** | Rótulo da unidade na página de Controle identificável | Validar em instalações autorizadas e garantir contexto da sessão; não infere permissão nem troca unidade | [contrato](../contracts/cli/unidade-atual.md) |
| `ies status` | **LOCAL**, diagnóstico Web limitado | Metadados/validação local de arquivos; GET opt-in do Controle | Não é login, certificação da sessão remota ou teste de permissões; evitar expandir flags antes das capacidades | [código](../src/status_cli.rs) |
| `ies setup` | **LOCAL** | Registra caminhos privados sem copiar credenciais | Não automatiza login, renovação ou troca de unidade; ampliar apenas se indispensável às operações já declaradas | [código](../src/configuracao.rs) |
| `ies esquema` | **PARCIAL LOCAL** | Lista nomes de comandos e versão | Entregar contrato/esquema útil se necessário, usando o comando existente; não criar outro inventário de comandos | [código](../src/main.rs) |
| `ies replay CENARIO` | **LOCAL** no acervo suportado | Reproduz capturas/cassettes verificadas offline: caixa, pesquisa, documento e histórico | É oráculo de testes, não substitui implementação Web dos comandos acima | [guia](replay.md) |

**Atualização #137:** a visão detalhada D do GET existente pode produzir
lista completa apenas com cobertura declarada demonstrada; continua um GET,
zero POST e **sem ensaio live autorizado**. Persistem 11 caminhos
incompletos e 3 locais; nenhum virou capacidade Web integral. A #57 segue
aberta para a paginação ativa.

**PR #138 PENDENTE, NÃO CONTABILIZADA NA MAIN:** a ordem de pré-validação
de `--pagina-arquivo` é corrigida para recusar complementos locais
inviáveis antes do GET potencialmente administrativo. A soma de 48 MB é
reverificada após o GET, sem reabrir os arquivos. Para suplemento inválido
conhecido, a proposta reduz até 1 GET evitável para 0; para complemento
válido conserva até 1 GET/0 POST. Não habilita paginação ativa nem muda
`PARCIAL WEB`; sem CI e revisão da PR, não se atribui entrega.

**Critério de contagem:** 14 caminhos operacionais, desconsiderando
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
