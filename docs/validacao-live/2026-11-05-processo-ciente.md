---
type: SEIObservation
title: Estado de validação de ciência de processo e documento em 05/11/2026
description: Evidências offline e pendências live da issue 271
---

# Ciência — validação e resíduos (05/11/2026)

| Ensaio | Status | Requisições |
| --- | --- | --- |
| Testes do leitor sintético, RED | FAIL como esperado | 0 SEI |
| Testes sintéticos e integração na main (fatia anterior) | PASS no CI da PR #280; `processo ciente` tem 3 GET de prévia e 7 GET no caso feliz sintético | 0 SEI |
| RED da regressão causa do GET mutador — [42215714854](https://github.com/franklinbaldo/ies/actions/runs/42215714854) | FAIL esperado: **3 falhas novas, 1.711 PASS, 22 ignorados preexistentes**; as falhas são HTTP 403, Infra 200 e transporte (testes antes do código) | 0 SEI |
| GREEN da preservação de causa | CI do novo head a verificar; não declarar PASS sem conclusão | 0 SEI |
| Replay íntegro das cassettes do Todos | NOT_RUN | 0 SEI |
| Prévia UnB (treinamento) | NOT_RUN | 0 |
| Prévia em segunda instalação (treinamento) | NOT_RUN | 0 |
| `--confirmar` em treinamento | NOT_RUN | 0 |
| Releitura de confirmação ao vivo | NOT_RUN | 0 |
| Escritas e resíduos novos | 0 / 0 | 0 |

Nenhuma escrita foi executada e **nenhum resíduo foi deixado no SEI**.
A execução ao vivo exige acesso comprovado a instalação de
treinamento, prévia no próprio processo, um único `--confirmar`
e verificação por leitura. Não usar produção. O CI só prova o
comportamento sintético, não a compatibilidade ao vivo.

## Aceite JS inerte — regressão da 🦊 Raposa (06/11/2026)

- **RED:** [run 42218975613](https://github.com/franklinbaldo/ies/actions/runs/42218975613), 3 FAIL novos, 1.714 PASS, 22 ignored preexistentes. Um comentário de bloco, um literal de string e um script em template eram interpretados incorretamente como atividade confirmada.
- **GREEN:** CI do head atualizado da PR #317 a conferir. A correção reaproveita o lexer estático do leitor documental para descartar chamadas dentro de comentário/literal e recusa scripts em template, sem executar JS.
- **Requisições SEI nesta rodada:** 0. Prévia UnB NOT_RUN (0); segunda instalação NOT_RUN (0); confirmação NOT_RUN (0); releitura live NOT_RUN (0); replay específico NOT_RUN. **Zero escritas e zero resíduos.**


## Regressão de controle de fluxo JS — 🦊 Raposa (06/11/2026)

- **RED antes do código:** [CI 42223177117](https://github.com/franklinbaldo/ies/actions/runs/42223177117) — **3 FAIL novos / 1.717 PASS / 22 ignored preexistentes**. Os testes provaram que `if(false)` e função arbitrária não invocada eram aceitos como atividade; o fluxo podia retornar sucesso se a lista relida já trouxesse o ID.
- **Correção:** só instrução independente no topo do script ou diretamente na função SEI `inicializar()` conta como aceite. Condições sem prova são recusadas. Não há execução de JS, regex, retry ou requisição adicional.
- **GREEN:** [CI 42224041949](https://github.com/franklinbaldo/ies/actions/runs/42224041949), head `e646671`, **5/5 PASS**: prek (Rustfmt, Clippy, suíte Rust, OKF, 20 KB, subcódigos), release e três replays gerais. O replay específico de ciência permanece NOT_RUN.
- **Orçamento sintético do caso adversarial:** 3 GET de abertura + 1 GET de tela + 1 GET potencialmente mutador + 1 GET de releitura = **6 GET, exatamente 1 mutador e zero retries**; não consultar lista sem atividade publicada por aceite válido.
- **Live:** UnB NOT_RUN (0 requisições); segunda instalação NOT_RUN (0); confirmação única NOT_RUN (0); verificação por leitura NOT_RUN (0). Replay específico de ciência NOT_RUN. **Nenhum acesso SEI, 0 escritas e 0 resíduos.**


## Regressão de retorno antes do aceite — 🦊 Raposa (06/11/2026)

- **RED real anterior à correção:** [run 42231850140](https://github.com/franklinbaldo/ies/actions/runs/42231850140), **2 FAIL novos, 1.724 PASS, 22 ignored preexistentes**. Um `return;` antes da recarga falsa ainda devolvia o ID `42457` e o fluxo anunciava ciência confirmada com base na lista antiga. Rustfmt dos dois testes novos também falhou no RED; formatação corrigida sem remover testes.
- **Correção:** aceitar `atualizarArvore` somente como primeira instrução executável no script ou diretamente no corpo de `inicializar()`, recusando código anterior cujo fluxo não foi comprovado. Não executar JS nem fazer retry.
- **Orçamento sintético exigido:** **6 GET / 1 GET mutador / 0 retry**, incluindo releitura independente da árvore. Sem ID do aceite, não buscar a lista preexistente. CI GREEN do novo head deve ser conferido separadamente.
- **Live:** UnB **NOT_RUN (0)**; segunda instalação **NOT_RUN (0)**; confirmação única **NOT_RUN (0)**; verificação de leitura **NOT_RUN (0)**; replay específico de ciência **NOT_RUN**. **0 requisições reais SEI, 0 escritas e 0 resíduos**.

## Checkbox em template na lista — 🦊 Raposa (06/11/2026)

- **RED antes da correção:** [CI 42240810943](https://github.com/franklinbaldo/ies/actions/runs/42240810943), **1 teste novo FAIL / 1.732 PASS / 22 ignored preexistentes**. A leitura real do DOM devolvia `Ok(true)` para atividade `42457` publicada somente dentro de `<template>` inerte, enquanto a linha efetiva trazia outra atividade. Rustfmt também apontou uma linha em branco excedente no teste novo, corrigida sem omitir o RED.
- **Correção:** `atividade_na_lista` recusa uma tabela `tblAndamentos` com descendente `<template>`, antes de contar checkboxes. Falha fechada com a causa existente `lista_de_ciencias_fora_da_forma_observada`; sem novos GETs, JavaScript, regex ou alteração no fluxo de escrita.
- **GREEN:** exigir CI verde no head final desta PR; nenhum PASS presumido.
- **Live:** UnB **NOT_RUN (0 requisições)**, segunda instalação **NOT_RUN (0)**, confirmação **NOT_RUN (0)**, verificação **NOT_RUN (0)** e replay específico **NOT_RUN**. **0 requisições reais ao SEI, 0 escritas, 0 resíduos**.



## Script de dados ou externo no aceite — 🦊 Raposa (06/11/2026)

- **Observação:** o leitor existente de `documento_ciencia` já distingue
  `script` executável de `script type=application/json` ou com `src`;
  o leitor do aceite ignorava esses atributos e podia atribuir uma atividade
  anterior à tentativa atual. As cassettes do acervo foram consultadas
  apenas por metadados; replay específico ainda não foi executado.
- **RED real antes da implementação:** [CI 42268070022](https://github.com/franklinbaldo/ies/actions/runs/42268070022):
  **1 FAIL / 1.739 PASS / 22 ignored preexistentes**; um script de dados
  forneceu `Ok("42457")` indevidamente, contra `AceiteSemAtividade`.
- **Correção:** em `src/ciencia.rs`, ler `type` e `src` por
  `html_atributos` antes de extrair `id_atividade`; ignorar scripts
  de dados ou com fonte externa, sem interpretar/executar JS. O teste
  de fluxo também protege contra a releitura de ciência antiga.
- **GREEN:** [CI 42268811765](https://github.com/franklinbaldo/ies/actions/runs/42268811765),
  head `9b72e70`, **5/5 PASS** (prek, binário e três replays gerais).
- **Orçamento sintético:** **6 GET, 1 potencialmente mutador, 0 retry**
  e **saída 3** quando o aceite só existe em script inerte. Uma releitura
  da árvore é feita, sem requisitar lista da atividade não comprovada.
- **Live:** prévia UnB **NOT_RUN (0 requisições)**; segunda instalação
  **NOT_RUN (0)**; `--confirmar` **NOT_RUN (0)**; releitura live
  **NOT_RUN (0)**; replay específico de ciência **NOT_RUN**.
  **0 requisições reais, 0 escritas e 0 resíduos novos**.


## Segundo argumento impede recarga — 🦊 Raposa (06/11/2026)

- **Observação pré-RED:** formas de aceite nos testes do Todos
  @323eb01 usam apenas uma URL literal em atualizarArvore.
  O leitor IES admitia segundo argumento sem evidência de execução.
- **RED real:** [CI 42278019363](https://github.com/franklinbaldo/ies/actions/runs/42278019363):
  **2 FAIL, 1.766 PASS e 22 ignorados preexistentes**. Com segundo
  argumento que lança exceção, o leitor devolveu Ok("42457")
  indevido e o fluxo consultou a lista além do roteiro de seis GET.
- **Correção:** reconhecer apenas chamada com um argumento literal.
  Exigir efeito incerto (saída 3), **6 GET, 1 GET mutador, zero retry**,
  sem consultar lista antiga sem atividade comprovada.
  GREEN no CI do head final a conferir.
- **Live:** UnB NOT_RUN (0 requisições), segunda instalação
  NOT_RUN (0), confirmação única NOT_RUN (0), releitura live
  NOT_RUN (0), replay específico NOT_RUN.
  **Zero requisições reais, zero escritas e zero resíduos** nesta sessão.
