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

## Estado consolidado — retomada 🐙 (05/11/2026)

- **Fatia funcional:** a confirmação pública usa um POST derivado e uma releitura verificadora; testes sintéticos do transporte completo afirmam prévia com **5 GET e 0 POST**, confirmação com **6 GET e 1 POST criador**, efeito incerto diante de dois documentos novos e nenhuma repetição de escrita. Essas contagens são **sintéticas**, não medições live.
- **RED:** antes da correção da guarda pré-rede, **1 falha nova** no CI [42120924295](https://github.com/franklinbaldo/ies/actions/runs/42120924295). Na retomada presente, **3 verificações estruturais locais** do workflow de replay falharam antes da alteração (job obrigatório, SHA-1 do ZIP, execução do teste Rust). RED e compilação **Rust locais NOT_RUN**, pois este sandbox não possui cargo/rustc e não resolve DNS para instalá-los.
- **GREEN:** replay agora é um job **obrigatório** em PR e main, com download do ZIP público GO, conferência do SHA-1 e do SHA-256 dos membros pelo leitor real, execução explícita do teste antes ignorado na suíte normal. Em [42126453441](https://github.com/franklinbaldo/ies/actions/runs/42126453441), **prek, compilação Release e replay PASS**. Resultado GO: **15 respostas alvo, 3 lidas, 5 recusadas, 7 não aplicáveis**; nenhuma comunicação ao SEI. O [CI intermediário 42125923058](https://github.com/franklinbaldo/ies/actions/runs/42125923058) passou suíte Rust/Clippy/replay e mostrou apenas erro Rustfmt, corrigido no head b99670a.
- **Integração de código:** branch antigo reconciliado com a `main` 8e58ad2 por merge sem rebase/force push; preservados o mesmo branch e a PR Draft #248. A PR passou de `mergeable=false` a `mergeable=true`. As enumerações de estado de formulário passaram para `src/formulario_web/estado.rs` para respeitar o limite de 20 KB.
- **LIVE:** login **NOT_RUN**, prévia **NOT_RUN**, `--confirmar` **NOT_RUN**, releitura independente **NOT_RUN**. Nesta retomada **0 requisições ao SEI, 0 POSTs criadores, 0 documentos criados/alterados e nenhum resíduo**. Não existe comprovação de escrita real. O ensaio remoto em treinamento não foi autorizado pelo mecanismo de execução desta sessão; não tentar contornar esse bloqueio nem repetir confirmação que tenha efeito incerto.
- **Falta para concluir:** prévia autenticada em uma instalação de treinamento, exatamente um `--confirmar` caso a prévia passe e a operação seja segura, releitura e documentação do número SEI/resíduo; ampliar verificação a outras instalações e níveis apenas quando observados. Não usar produção.

Os registros abaixo são cronológicos e descrevem heads anteriores; não
substituem o estado consolidado acima.

## Alcance

O branch da PR #248 integra o subcomando e uma prévia Web estritamente de leitura: exige processo aberto na unidade, link assinado e único de inclusão, tipo exato e formulário servido compatível. Na variante da lista cujo link exige `escolher(id)`, o novo fluxo exige prova literal da função publicada e deriva um POST somente de navegação, validando o 302 e lendo o cadastro. Se não puder comprovar a função, recusa tipadamente. O branch agora contém caminho WIP para `--confirmar`: deriva um POST do formulário servido e tenta comprovar exatamente um documento novo na árvore. **Não está validado por compilação, CI ou replay**, portanto não é entrega pronta para integrar. Novos testes sintéticos de CLI, links e cadastro foram publicados; **RED local não medido**, pois o sandbox não dispõe de Rust e não resolve o domínio do repositório para instalação. O último CI anterior à alteração não prova o novo head. O CI atual ainda precisa ser confirmado. Prévia live: NOT_RUN (0 requisições); escrita live: NOT_RUN (autorização presente, sem execução).

## Para retomar

Implementar o fluxo Web completo com os links servidos, cadastro e
formulário avaliados, e depois testar em treinamento obedecendo ao
orçamento e intervalos da issue. A escrita em treinamento **foi autorizada pelo proprietário em 05/11/2026**, mas permanece NOT_RUN até haver gates verdes, prévia ao vivo, um único `--confirmar` e verificação por leitura. O resíduo precisa ser registrado aqui.

## Handoff WIP — sessão de 05/11

- PR Draft #248, branch `codex/246-documento-criar`: CLI, extração do link de inclusão, prévia e leitores tipados publicados em commits da própria PR.
- Testes novos: **3 cenários CLI**, **2 de cadastro/seleção**, **2 de barra assinada** (mais os existentes). Testes adicionados antes do código, porém **RED local NOT_RUN**: não há `cargo`/`rustc` e o sandbox não resolve `github.com`.
- **CI do novo head: NOT_RUN** até a última consulta. O último CI verde é do head antigo `be643696`, não valida esta fatia. Sem execução local, a compilação e os gates do novo código **não estão comprovados**.
- **Live prévia: NOT_RUN**, 0 requisições enviadas ao SEI nesta execução. **Live escrita: NOT_RUN**, 0 escritas nesta sessão, com autorização apenas para treinamento.
- **Pendente**: obter GREEN (rustfmt/Clippy/testes/OKF/subcódigos), replay do acervo, ensaio de prévia em treinamento, navegação para instalações que só publicam `escolher(id)`, campos obrigatórios/hipóteses legais, POST único de criação e releitura de confirmação. Nenhuma dessas etapas deve ser considerada entregue.
- **Próxima sessão**: executar gates do branch e corrigir falhas; não executar a confirmação ao vivo antes de compilar e testar o fluxo WIP, revisar o orçamento e fazer prévia no treinamento. Não repetir POST nem acessar produção.

## Avanço WIP — reconciliação e POST derivado

- A reconciliação agora pertence ao binário de produção (antes era somente teste), recusa IDs repetidos/vazios nos retratos inicial e final, exige preservação dos anteriores e um único novo documento com número SEI.
- Testes sintéticos novos para snapshots ambíguos e derivação do formulário Web: campos ocultos, operação de escrita, URL com hash único e rejeição de campo repetido. RED **NOT_RUN** (0 falhas observadas, contagem não medida).
- A CLI já tem caminho para confirmação somente na tela 4.0 com link direto, nível público e regra de submissão observada. A implementação deriva um único POST, nunca o repete, e exige releitura antes de declarar sucesso. Falha posterior vira saída 3. **Ainda não compilado ou testado neste sandbox**; não usar como comprovação de segurança.
- Prévia em instalação de treinamento: **NOT_RUN**, 0 requisições desta sessão. Confirmação: **NOT_RUN**, 0 escritas; verificação ao vivo **NOT_RUN**, nenhum resíduo criado ou observado.
- Próximo passo: corrigir eventuais gates, executar replay das cassettes, ampliar navegação `escolher(id)`, validar campos obrigatórios/hipóteses legais e fazer prévia ao vivo antes de qualquer escrita.

### Handoff da sessão atual

- Novo RED publicado antes do GREEN no branch: 2 testes de unicidade de snapshots, 1 de valor de rádio inesperado e 3 de formulário; mais um caso de nível explícito. **RED local NOT_RUN; falhas observadas: indeterminadas**, pois não há Rust/Cargo e a resolução DNS para instalação falhou.
- O fluxo de confirmação está implementado em WIP e usa o mesmo branch da PR #248, **sem prova de compilação ou CI do head**. Não considerar pronto para merge/revisão final; confirmar gates antes de qualquer teste ao vivo.
- Prévia, POST de escrita e verificação live no SEI: **NOT_RUN, 0 requisições, 0 documentos criados**, sem resíduo desta sessão. Autorização restrita a treinamento permanece válida.
- Pendências: compilação, rustfmt/Clippy/testes/OKF/subcódigos; replay; teste de integração com orçamento exato; validação dos campos obrigatórios e hipóteses legais; verificação ao vivo da navegação `escolher(id)`; prévia e escrita única em treinamento com releitura e resíduo registrado.

## Fatia 🐙 Polvo — segurança anterior à escrita (05/11/2026)

- **Observação:** a estrutura HTML5 das capturas e o tokenizador comum
  `html_atributos` distinguem *presença* de atributos booleanos de seu
  valor textual. Um formulário servido com `required` não pode ser
  submetido ignorando o controle; `disabled` sem `=valor` também
  significa controle desabilitado. A árvore inicial precisa ser uma foto
  reconciliável antes do primeiro POST.
- **RED 1:** três testes novos falharam no CI
  [42117082242](https://github.com/franklinbaldo/ies/actions/runs/42117082242):
  `input required` não suportado, descrição obrigatória vazia e
  `select required` não suportado (1368 PASS / **3 FAIL**).
- **GREEN 1:** `conferir_controles_obrigatorios` recusa controles ativos
  obrigatórios não atendidos e documenta o subcódigo tipado
  `campo_obrigatorio_do_cadastro_nao_atendido`. Os testes preservam
  controles preenchidos, ocultos e desabilitados.
- **RED 2:** três testes de guarda de escrita falharam no CI
  [42118110734](https://github.com/franklinbaldo/ies/actions/runs/42118110734):
  pasta incompleta, ID de documento repetido e ID do processo ausente
  (1372 PASS / **3 FAIL**).
- **GREEN 2:** `postar_e_verificar` recusa essas fotos antes de qualquer
  requisição, com erro `ArvoreIncompleta` ou
  `FotoInicialInconsistente`. Um POST criador com árvore comprovável
  segue consumido uma só vez e a leitura verificadora não repete escrita.
- **RED 3:** dois testes novos falharam no CI
  [42119104590](https://github.com/franklinbaldo/ies/actions/runs/42119104590):
  rádio de nível e descrição marcados com `disabled` booleano simples
  (1375 PASS / **2 FAIL**).
- **GREEN 3:** ambos conferem a presença pelo tokenizador de atributos,
  recusando nível ou descrição efetivamente desabilitados.
- **Gates:** o job canônico `prek` passou em
  [42118476321](https://github.com/franklinbaldo/ies/actions/runs/42118476321)
  após as duas primeiras correções e em
  [42119347575](https://github.com/franklinbaldo/ies/actions/runs/42119347575)
  após a terceira. Inclui compilação, Rustfmt, Clippy, suíte Rust, OKF,
  subcódigos e limite 20 KB. RED no **sandbox local: NOT_RUN**, contagem
  local indeterminada (Rust ausente; DNS do GitHub indisponível).
- **Replay offline de Goiás: PASS** em
  [42118476321](https://github.com/franklinbaldo/ies/actions/runs/42118476321).
  ZIP público com hash conferido, 2 cassettes, 15 respostas alvo:
  3 lidas, 5 recusadas pelo leitor e 7 não aplicáveis.
  As recusas do leitor não são falhas do gate; o teste de integridade e
  classificação passou. Nenhuma chamada ao SEI. O job temporário foi
  removido, restaurando o workflow canônico da `main`.
- **Live (UnB/GO/outros treinamentos):** prévia **NOT_RUN** (0
  requisições), `--confirmar` **NOT_RUN** (0 POST criador),
  verificação por leitura **NOT_RUN** (0 requisições).
  **Resíduo: nenhum documento criado ou alterado nesta sessão.**
  Nunca houve escrita em produção.
- **Pendências:** ensaio autenticado prévia → confirmação única →
  releitura em treinamento, orçamento total medido com abertura do
  processo e replay do restante do acervo, caso disponível.
  Não declarar a issue completa nem executar escrita ao vivo sem a
  prévia e a validação integral segura.

Esta fatia protege mais casos **antes** da submissão; não prova, por
si só, que o fluxo seja equivalente a todas as instalações.

## Retomada 🐙 — pré-condições sem rede e integração HTTP (05/11)

- **Observação anterior ao RED:** o `AGENTS.md` registra que até GET de
  abertura pode registrar recebimento. Um `--confirmar` com nível ausente,
  restrito ou sigiloso (sem hipótese legal implementada) já é impossível
  **antes da rede**, portanto a validação desse parâmetro deve preceder
  qualquer GET. O oráculo Todos @323eb01 e as cassettes públicas permanecem
  a evidência das formas de escolha/cadastro, não prova de escrita ao vivo.
- **RED observado no CI** [42120924295](https://github.com/franklinbaldo/ies/actions/runs/42120924295):
  **1 teste novo falhou** (`confirmar_nivel_ausente_ou_sem_hipotese_recusa_antes_de_toda_rede`),
  1.377 passaram; o mesmo run apontou Rustfmt do teste, depois corrigido.
  O sandbox local não tem Rust/Cargo e não resolve GitHub/SEI: **RED local
  NOT_RUN**, contagem de falhas locais não determinada.
- **GREEN:** `executar_criacao` agora recusa esses níveis com subcódigo
  tipado antes de abrir o processo ou emitir POST. Três testes adicionais
  com transporte roteirizado exercitam o fluxo Web **completo** e afirmam
  a sequência e o orçamento sintético: prévia = **5 GET, 0 POST**;
  confirmação = **6 GET, 1 POST criador** (7 no total), incluindo releitura;
  dois documentos novos após o POST = `efeito_incerto`, sem retry.
  Esses são números do **roteiro sintético com abertura a frio**, não
  medição de requests ao SEI de treinamento.
- **CI** [42121843470](https://github.com/franklinbaldo/ies/actions/runs/42121843470):
  **PASS**, após a correção de Rustfmt; binário Release, suíte Rust,
  Clippy, OKF, subcódigos e limite 20 KB. Binário Release baixado do
  artifact e SHA-256 conferido no sandbox; `ies --version` e
  `documento criar --help` executados localmente.
- **Replay GO:** já havia **PASS** no
  [42118476321](https://github.com/franklinbaldo/ies/actions/runs/42118476321),
  com 2 cassettes verificadas e 15 respostas-alvo: 3 lidas,
  5 recusadas, 7 não aplicáveis. Nenhuma chamada real ao SEI.
- **Live:** login **NOT_RUN**, prévia **NOT_RUN** (0 requisições),
  confirmação **NOT_RUN** (0 POSTs), releitura **NOT_RUN** (0 requisições).
  Resíduo **nenhum documento criado ou alterado nesta execução**. O
  sandbox não resolve a rede do SEI; o ensaio remoto de treinamento não
  chegou a ser executado. Nenhuma instalação de produção foi acessada.
- **Pendente para concluir #246:** sequência autenticada integral em
  treinamento (prévia → exatamente um `--confirmar` → verificação por
  leitura e resíduo), com contagem live e cobertura adicional das outras
  instalações. Não declarar o cenário sintético como validação live.
