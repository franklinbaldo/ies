---
type: Guide
title: Validação da criação de documento interno (issue 246)
description: Resultado das verificações e dos passos ainda não executados
---

# Documento criar — 05/11/2026

Operação: `documento.criar`, issue
[246](https://github.com/franklinbaldo/ies/issues/246).

| Ensaio | Resultado | Requisições |
| --- | --- | --- |
| Leitura das fontes do Todos e das capturas | PASS documental | 0 |
| Rustfmt, Clippy, testes Rust, OKF, subcódigos, 20 KB e binário Release | PASS no CI 42126453441 (head b99670a) | 0 ao SEI |
| Replay obrigatório de 2 cassettes GO com hashes conferidos | PASS no CI 42126453441; 15 respostas-alvo | 0 ao SEI |
| Replay das demais cassettes/instalações | NOT_RUN | 0 ao SEI |
| Prévia Web em treinamento | NOT_RUN | 0 ao SEI |
| Criação ao vivo, releitura e resíduo no SEI | NOT_RUN — autorizada somente em treinamento | 0 ao SEI |

## Diagnóstico UnB 06/11 — sem POST

[Run 42222453651](https://github.com/franklinbaldo/ies/actions/runs/42222453651),
somente TREINAMENTO: login **PASS (6 requisições)**; prévia **PASS (4)**;
diagnóstico `--confirmar` **FAIL pré-POST**, subcódigo
`submissao_do_documento_nao_observada`, requisições **null**.
**0 POST criador, 0 escrita, 0 resíduo**; releitura **NOT_RUN**.
Botões Salvar ativos; `tl::inner_text` extraiu só 15 bytes do script
ativo 24, mas leitura independente encontrou a cadeia estática.
HTML apenas em arquivo privado temporário, já descartado; workflow
e captura temporários removidos. O teste sintético em `54a7ac1`
passou sem RED; correção de extração requer CI e novo ensaio seguro.

## Ensaio autenticado UnB após correção da escolha (06/11/2026)

- Execução **somente em treinamento**, via [workflow temporário
  42198707929](https://github.com/franklinbaldo/ies/actions/runs/42198707929)
  (commit de execução `d3d6127`). O workflow foi **removido** do branch no
  commit `3bd0f1d`; nenhuma automação de escrita ficou habilitada.
- **Login PASS**, 5 requisições reais. **Prévia PASS**, 4 requisições reais,
  com o NUP `23106.001868/2026-91`, tipo exato `Despacho`, unidade
  publicada, e `confirmado=false`, `alterado=false`. A correção da URL
  do tipo duplicada superou a recusa anterior na prévia.
- Após a prévia, houve **uma única invocação** de `--confirmar`, que
  retornou **FAIL (saída 1)** com
  `submissao_do_documento_nao_observada`. A guarda de
  `src/documento_criar/fluxo.rs` devolve esse erro **antes** da função
  `postar_e_verificar`: **0 POSTs criadores**, nenhuma escrita por este
  comando. A contagem HTTP da invocação recusada **não foi publicada**
  (`null`); não deduzir zero requisições de leitura/navegação.
- A releitura adicional independente executou **PASS (1 requisição)**,
  mas não comprovou documento novo. O roteiro classificou o resíduo como
  **FAIL/readback_unproven** por não existir confirmação, não como prova
  de escrita. **Nenhum número SEI ou resíduo novo comprovado**.
- Orçamento real observado: login **5**, prévia **4**, confirmação
  **não observada (null)**, releitura **1**; total real completo
  **não observado**. Nenhum retry de escrita; a confirmação não chegou ao
  POST criador. A nova barreira funcional é a **forma do script de Salvar
  servido pela UnB**, não a seleção do tipo.
- **Próximo passo:** observar com segurança a estrutura de funções e botões
  do cadastro *real* (sem divulgar cookie/URL assinada/infra_hash), testar
  primeiro um adversarial RED, adaptar a guarda apenas à forma comprovada,
  revalidar CI e repetir **somente a prévia**, antes de decidir por outra
  confirmação. Não presumir que o JavaScript do acervo seja igual ao
  cadastro atual e nunca enfraquecer a recusa pré-POST.

## Continuidade 🐙 — scripts inertes e integração com a main (05/11/2026)

- **Observação pré-RED:** `script[src]` ignora o conteúdo inline; um
  template literal JS multilinha contém texto, não prova execução da linha.
- **RED:** [CI 42146255058](https://github.com/franklinbaldo/ies/actions/runs/42146255058), head `d2567d1`:
  **1 teste novo FAIL**, 1.584 PASS, 16 ignorados. Testes com transporte
  sintético, **5 GET e 0 POST** na recusa; nenhuma escrita real.
- **GREEN antes da reconciliação:** [CI 42146635812](https://github.com/franklinbaldo/ies/actions/runs/42146635812),
  head `29157e3`, 3/3 jobs PASS. A guarda rejeita `script[src]`
  e literais JS inertes sem executar JS; nenhuma mudança na CLI pública.
- **Merge no branch da PR:** `e06878e` incorpora a main `2d05da9`
  sem rebase nem force-push, conciliando `ErroHtml` e preservando
  as alterações de outras frentes. Rustfmt do merge será corrigido.
- **LIVE:** login, prévia, `--confirmar` e releitura **NOT_RUN**;
  **0 requisições reais, 0 escritas, nenhum resíduo**. Treinamento
  continua pendente; jamais tentar confirmar sem prévia e gates verdes.

## Retomada 🐙 — script ativo e causas tipadas (05/11/2026)

- **Observação antes dos testes:** a guarda de confirmação anterior
  aceitava a sequência de `hdnFlagDocumentoCadastro` como qualquer
  substring do HTML, inclusive em `template`, `script type=application/json`,
  comentário JavaScript ou script duplicado. Essas estruturas não provam
  que o formulário Web ofereceu a instrução ativa. Nenhum JS foi executado.
- **RED/ordem:** primeiro commit `772bac7` com quatro fixtures
  sintéticas adversariais, depois `7a32f7c` com a correção. Primeiro
  commit `f015675` exigindo composição de `Causa`, depois migração
  dos erros. Teste adicional de preservação do HTTP 200 da Infra em
  `bf6c27b` antes do ajuste `aae94d0`. **RED isolado desses testes:
  NOT_RUN (falhas não aferidas)**, pois nenhuma execução CI foi
  observada nesses heads intermediários; não se infere contagem.
- **Implementação:** o leitor do cadastro agora exige uma única
  instrução literal em elemento `script` ativo reconhecido, não
  texto inerte ou duplicado, e recusa antes de enviar POST. Os erros
  de seleção, barra e prévia usam `#[derive(Causa)]`, preservam
  subcódigos anteriores, composição tipada, status e mensagem HTTP
  conhecidos; o erro após POST continua com saída 3.
- **CI:** [run 42140055432](https://github.com/franklinbaldo/ies/actions/runs/42140055432)
  observou suíte Rust **PASS** e binário Release **PASS**; `prek`
  **FAIL** por Rustfmt, métodos não usados em Clippy e checkout sem
  `origin/main` para a auditoria `wip-ignores`. Correções subsequentes
  de formatação, remoção de wrappers mortos e checkout com histórico
  completo foram publicadas no mesmo branch; **CI final ainda a
  verificar**. Não se desativou nem enfraqueceu um teste/gate.
- **LIVE (esta retomada):** login **NOT_RUN**; prévia **NOT_RUN**;
  `--confirmar` **NOT_RUN**; verificação independente **NOT_RUN**.
  **0 requisições ao SEI, 0 POSTs criadores, 0 documentos criados ou
  alterados e nenhum resíduo.** Não houve acesso a produção nem
  tentativa de escrita em treinamento sem prévia e gates seguros.
- **Pendente:** CI integralmente verde no head final; só depois,
  prévia autenticada em TREINAMENTO → uma confirmação única se
  for segura → releitura independente com número SEI, orçamento
  de requisições real e resíduo. Não repetir POST de efeito incerto.

## Continuidade 🐙 — 302 sem Location (05/11/2026)

- **Observação anterior ao RED:** `enviar_submissao_via` trata 302 como dado e não segue redirecionamentos. Um 302 **sem Location** não comprova consequência da escrita; outra sessão poderia ter alterado a árvore entre as leituras.
- **RED observado:** commit de teste `7f0ab34` precedeu a correção. [CI 42131591459](https://github.com/franklinbaldo/ies/actions/runs/42131591459): `prek` FAIL; **1 teste novo falhou**, 1.610 aprovados, 17 ignorados. O teste exige `EfeitoIncerto`, com **1 POST + 1 releitura**, sem retry. Release PASS.
- **GREEN a verificar:** no commit `bda36d9`, `postar_e_verificar` só aceita 302 com Location não vazia. Mantém a leitura única mesmo após efeito incerto. O CI anterior [42127145672](https://github.com/franklinbaldo/ies/actions/runs/42127145672) era PASS mas não cobre a nova correção. Rust/Cargo e gate local NOT_RUN por falta de toolchain e DNS.
- **Live:** login NOT_RUN; prévia NOT_RUN; confirmação NOT_RUN; releitura NOT_RUN; **0 requisições reais, 0 POST criador, nenhum resíduo**. Sandbox sem acesso à instalação de treinamento; nenhuma execução em produção.

## Estado consolidado — retomada 🐙 (05/11/2026)

- **Fatia funcional:** a confirmação pública usa um POST derivado e uma releitura verificadora; testes sintéticos do transporte completo afirmam prévia com **5 GET e 0 POST**, confirmação com **6 GET e 1 POST criador**, efeito incerto diante de dois documentos novos e nenhuma repetição de escrita. Essas contagens são **sintéticas**, não medições live.
- **RED:** antes da correção da guarda pré-rede, **1 falha nova** no CI [42120924295](https://github.com/franklinbaldo/ies/actions/runs/42120924295). Na retomada presente, **3 verificações estruturais locais** do workflow de replay falharam antes da alteração (job obrigatório, SHA-1 do ZIP, execução do teste Rust). RED e compilação **Rust locais NOT_RUN**, pois este sandbox não possui cargo/rustc e não resolve DNS para instalá-los.
- **GREEN:** replay agora é um job **obrigatório** em PR e main, com download do ZIP público GO, conferência do SHA-1 e do SHA-256 dos membros pelo leitor real, execução explícita do teste antes ignorado na suíte normal. Em [42126453441](https://github.com/franklinbaldo/ies/actions/runs/42126453441), **prek, compilação Release e replay PASS**. Resultado GO: **15 respostas alvo, 3 lidas, 5 recusadas, 7 não aplicáveis**; nenhuma comunicação ao SEI. O [CI intermediário 42125923058](https://github.com/franklinbaldo/ies/actions/runs/42125923058) passou suíte Rust/Clippy/replay e mostrou apenas erro Rustfmt, corrigido no head b99670a.
- **Integração de código:** branch antigo reconciliado com a `main` 8e58ad2 por merge sem rebase/force push; preservados o mesmo branch e a PR Draft #248. A PR passou de `mergeable=false` a `mergeable=true`. As enumerações de estado de formulário passaram para `src/formulario_web/estado.rs` para respeitar o limite de 20 KB.
- **LIVE:** login **NOT_RUN**, prévia **NOT_RUN**, `--confirmar` **NOT_RUN**, releitura independente **NOT_RUN**. Nesta retomada **0 requisições ao SEI, 0 POSTs criadores, 0 documentos criados/alterados e nenhum resíduo**. Não existe comprovação de escrita real. O ensaio remoto em treinamento não foi autorizado pelo mecanismo de execução desta sessão; não tentar contornar esse bloqueio nem repetir confirmação que tenha efeito incerto.
- **Falta para concluir:** prévia autenticada em uma instalação de treinamento, exatamente um `--confirmar` caso a prévia passe e a operação seja segura, releitura e documentação do número SEI/resíduo; ampliar verificação a outras instalações e níveis apenas quando observados. Não usar produção.

Os registros abaixo são cronológicos e descrevem heads anteriores; não
substituem o estado consolidado acima.

## Histórico das etapas anteriores

O registro cronológico integral (incluindo WIP, limites e contagens) foi preservado em [histórico da implementação](2026-11-05-documento-criar-historico.md).

## Continuidade 🐙 — Location de host externo (05/11/2026)

- **Observação pré-RED:** `postar_e_verificar` já recusava 302 sem
  Location, mas aceitava qualquer cadeia não vazia, inclusive um
  redirecionamento para domínio estranho à instalação. Uma alteração
  concorrente da árvore podia então ser atribuída indevidamente ao POST.
- **Teste antes do código:** commit `61378b8` adicionou regressão sintética
  com Location externa, **um POST e um GET** de releitura, saída 3 exigida.
  O CI desse commit foi cancelado pela concorrência de novos pushes;
  **RED isolado não observado** (não declarar contagem de falhas).
- **Implementação:** commit `e00a5c5` restringe 302 a Location do
  mesmo controlador HTTPS com ação explícita. Commit `db3dd8a`
  adicionou controles positivos para Location absoluta e relativa locais.
  Nenhum redirect é seguido, nenhuma escrita é repetida.
- **GREEN do novo head:** ainda a verificar no CI `ci.yml`.
  **LIVE:** login, prévia, confirmação e releitura em treinamento
  **NOT_RUN** nesta execução; **0 requisições novas, 0 POST criador,
  0 resíduo**. Ensaio remoto com credenciais de treinamento não foi
  iniciado devido ao bloqueio do ambiente de execução.

## Continuidade 🐙 — guarda de ramo inatingível (05/11/2026)

- **Observação anterior ao RED:** no oráculo Todos
  `tests/paginas_reais/documento_gerar.html @323eb01`,
  a gravação é uma `function submeter()` com desabilitação,
  atribuição da flag e `frmDocumentoCadastro.submit()` em sequência
  direta. Uma instrução em `if (false)` não comprova escrita.
- **Teste publicado antes da implementação:** commit `58839c8` adicionou
  casos integrados que devem recusar antes do POST quando a instrução
  está em ramo inatingível, função não relacionada ou aninhamento.
  Commit `569e8cd` acrescentou caso positivo com o corpo do oráculo.
- **RED observado:** [CI 42158719053](https://github.com/franklinbaldo/ies/actions/runs/42158719053),
  **1 teste FAIL, 1.659 PASS, 18 ignorados**; a falha
  `script_inerte_ou_duplicado_nao_autoriza_post_criador` foi
  `requisição além do roteiro`, evidência de POST indevido no cenário
  roteirizado, sem qualquer contato real com o SEI.
- **Correção:** a guarda passa a exigir script inteiro mínimo ou corpo
  direto de `function submeter()` na ordem publicada; reconhecedor
  separado em `src/documento_criar/guarda_script.rs`, sem regex ou
  execução de JS. Os adversariais afirmam **5 GET / 0 POST**.
- **CI intermediário da implementação:** [run 42159144227](https://github.com/franklinbaldo/ies/actions/runs/42159144227)
  aprovou a suíte Rust, mas acusou Rustfmt e Clippy (função longa e
  constantes declaradas depois de instruções). O código foi modularizado
  sem desativar lint/teste, e novo CI é necessário no head final.
- **Live da execução:** login **NOT_RUN**, prévia **NOT_RUN**,
  `--confirmar` **NOT_RUN**, releitura independente **NOT_RUN**.
  **0 requisições ao SEI, 0 POSTs reais, 0 documentos ou resíduos.**
  Não houve acesso a produção; só se escreverá em treinamento após
  prévia autenticada e gates verdes.


## Continuidade 🐙 — vínculo de Salvar com a submissão (06/11/2026)

- **Observação pré-RED:** captura Todos `documento_gerar.html @323eb01`,
  linhas 810/1061: botões Salvar habilitados `button#btnSalvar`,
  `type=button`, `onclick=confirmarDados()`. Uma função `submeter()`
  sem esse controle pode ser inerte. Nota de observação publicada no
  commit `0c0a4d2` antes do teste.
- **RED:** [CI 42170278378](https://github.com/franklinbaldo/ies/actions/runs/42170278378),
  commit de testes `f16ee68`: **1 FAIL / 1.683 PASS / 18 ignorados**.
  O caso negativo ultrapassou as cinco GET sintéticas ao tentar
  prosseguir sem botão Salvar; não houve rede real.
- **Correção:** `guarda_script.rs` confere os botões servidos por
  `html_atributos` e recusa controle ausente, divergente, desabilitado
  ou duplicação contraditória. Dois controles iguais do topo/rodapé são
  legítimos. Testes de fronteira exigem **5 GET / 0 POST**, e os casos
  positivos mantêm **6 GET / 1 POST / 1 releitura**.
- **GREEN do código no head `4483953`:** [CI 42170922268](https://github.com/franklinbaldo/ies/actions/runs/42170922268), **5/5 jobs PASS**: prek (Rustfmt, Clippy, testes Rust, OKF, subcódigos, 20 KB, WIP), binário Release e replays GO de documento.criar, lupa e processo.criar. O CI imediatamente anterior [42170712812](https://github.com/franklinbaldo/ies/actions/runs/42170712812) teve **Clippy FAIL** apenas por dois identificadores sem crases no comentário novo; corrigido sem enfraquecer lint.
- **LIVE (esta sessão):** login **NOT_RUN**, prévia **NOT_RUN**,
  `--confirmar` **NOT_RUN**, leitura independente **NOT_RUN**;
  **0 requisições reais**, **0 POST criador**, **0 documentos criados ou
  alterados**, **nenhum resíduo**. Autorização restrita a TREINAMENTO;
  sem conectividade que permita sequência segura, não tentar escrita.

## Continuidade 🐙 — cadeia de confirmação (06/11/2026)

- **Observação anterior ao RED:** Todos `documento_gerar.html @323eb01`
  linhas 615–633: `confirmarDados()` → `OnSubmitForm()` →
  `submeter()`; botões Salvar nas linhas 810/1061.
- **RED:** testes primeiro no commit `d8e0461`, [CI 42176956473](https://github.com/franklinbaldo/ies/actions/runs/42176956473):
  **1 FAIL, 1.684 PASS, 18 ignorados**; tentou exceder os 5 GET
  sintéticos ao aceitar a função de submissão sem cadeia acionável.
- **Código:** guarda reconhece a cadeia inteira sem interpretar ou
  executar JS, com regressões **5 GET/0 POST** e controle positivo
  **6 GET/1 POST com releitura**.
- **Live TREINAMENTO:** login **NOT_RUN**, prévia **NOT_RUN**,
  `--confirmar` **NOT_RUN**, releitura **NOT_RUN**; **0 requisições
  reais, 0 escritas e nenhum resíduo**. Sem ambiente seguro para a
  sequência completa; nunca executar em produção.


## Ensaio de prévia em UnB de treinamento — 06/11/2026

Dois ensaios independentes executaram o binário Release do branch da
[PR #248](https://github.com/franklinbaldo/ies/pull/248), com arquivos
privados `0600`, credenciais públicas e **sem `--confirmar`**.
Cada login foi **PASS com 5 requisições reais**. Entre login e prévia,
houve pausa de 30 segundos. Não se repetiu uma requisição falhada.

| Ensaio | Login | Prévia | Requisições da prévia | POST criador | Resíduo |
| --- | --- | --- | --- | --- | --- |
| [42183949372](https://github.com/franklinbaldo/ies/actions/runs/42183949372), processo 23106.001875/2026-93 | PASS (5) | FAIL: `processo_nao_aberto_na_unidade_da_sessao` (saída 1) | **não observadas** (`null`) | 0 | nenhum |
| [42184286157](https://github.com/franklinbaldo/ies/actions/runs/42184286157), processo 23106.001868/2026-91 | PASS (5) | FAIL: `endereco_do_tipo_de_documento_nao_reconhecido` (saída 1) | **não observadas** (`null`) | 0 | nenhum |

O job do GitHub terminou com sucesso operacional porque o script
encerrou corretamente ao encontrar a recusa; **não é PASS funcional da
prévia**. Confirmar **NOT_RUN**, releitura **NOT_RUN**, escritas
**0**, documentos criados ou alterados **0**, resíduo **nenhum**.
O comando de erro não publicou `requisicoes`; não atribuir zero às
requisições de prévia.

A captura Todos `documento_escolher_tipo.html @323eb01` contém um
link de Despacho com dois `id_procedimento` idênticos. O leitor atual
recusa essa forma; isso é uma **hipótese sobre a segunda falha live**,
não uma prova da URL efetivamente servida na execução. O teste RED
[42184932636](https://github.com/franklinbaldo/ies/actions/runs/42184932636)
detectou a divergência, com **1 falha nova, 1.722 PASS, 19 ignorados**.
Não há autorização técnica para escrever antes de uma prévia completa
que comprove unidade, tipo e cadastro. O workflow temporário usado
nesses ensaios foi removido do branch.

Próximo ensaio: obter nova prévia de treinamento em processo aberto na
unidade da sessão, observar o link real de escolha sem expor assinatura,
e só então decidir sobre confirmação única segura. Nunca repetir uma
escrita incerta ou publicar URL assinada.

A retomada de 06/11 sobre controles inertes, RED e bloqueio live está documentada no [histórico de implementação](2026-11-05-documento-criar-historico.md).
