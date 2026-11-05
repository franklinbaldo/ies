---
type: ImplementationDecision
title: Criar processo pela escolha e cadastro publicados pelo SEI
operation: processo.criar
implementation: partial
status: adopted-for-scope
scope: escolha-de-tipo-e-cadastro-web-com-previa-e-verificacao-por-leitura
selected_strategy: web-formulario-servido-e-releitura-da-arvore
alternatives: web-formulario-servido-e-releitura-da-arvore,post-montado-estaticamente,rest-wssei
evidence_level: todos-captura-409-replay-publico-sem-live
live_validation: not-run
---

# Decisão proposta: iniciar processo (issue #245)

## Alternativas consideradas

O oráculo Todos demonstra criação em instalações de treinamento.
A lista de tipos pode ser parcial; o cadastro e o modo de escolha
dependem da versão do SEI. O comando desejado é
`ies processo criar --tipo TIPO --especificacao TEXTO`,
com prévia por padrão, `--nivel publico|restrito`,
`--interessado` repetível e `--confirmar`.

1. **Web, formulário servido, confirmação por leitura** (proposta).
   Usa ações e controles observados, uma escrita e a árvore relida;
   compatível com o primitivo `formulario_web`.
2. POST montado de campos fixos: menos passos, porém incapaz
   de garantir que o tipo, a hipótese e os hidden sejam oferecidos.
3. REST/WSSEI: sem prioridade no IES e não demonstrado no oráculo.

## Decisão

Seguir a alternativa 1; a contagem RED original não foi medida e não deve ser inventada.
Escolher exclusivamente um tipo demonstrável pela lista completa,
abrir o cadastro pela ação publicada e conferir campos,
assunto sugerido, nível e hipótese antes de escrever.
Na prévia, apresentar o plano e os campos efetivamente lidos,
sem POST criador. O POST de escolha de tipo, quando publicado
pela versão, somente abre o cadastro.

Com `--confirmar`, o primitivo deve enviar uma única vez o
formulário derivado, com efeito avaliado de criação de processo.
Desabilitar redirecionamento automático; obter identidade da
resposta e reler a árvore. Falha posterior à escrita cuja leitura
não confirme a identidade é efeito incerto (3), sem repetição.
Não tratar simples HTTP 200/302 como prova suficiente.

Não usar regex nem executar JavaScript. Extrair atributos
unicamente por `src/html_atributos`; nunca expor URLs assinadas,
cookies ou `infra_hash`. A escrita live está autorizada **somente em treinamento**, conforme
comentário da issue #245 em 05/11/2026; a autorização não equivale à
validação da capacidade. Executar prévia, uma confirmação e releitura.

## Critérios antes da adoção

- RED: tipo ausente na lista completa, duplicado, incompleta,
  sem ação conhecida, assunto ausente/múltiplo, rádio desabilitado,
  hipótese obrigatória ausente, interessado ambíguo.
- GREEN: prévia com zero POST criador; confirmação com exatamente
  um POST criador e verificação fresca da identidade.
- Respostas: 302 com ID e NUP coincidentes, 200 com recusa,
  falha de rede depois do POST, 302 sem ID, árvore indisponível.
- Registrar contagem RED antes da implementação; replay de
  cassettes e orçamento por versão; executar gate pre-push e CI.
- Prévia live e escrita live têm registros independentes
  PASS/FAIL/NOT_RUN. Na ausência de execução: NOT_RUN.

## Comparação

A implementação Rust na PR #249 contém a leitura da lista,
conferência do cadastro, consulta AJAX de interessados, prévia Web,
POST de criação único, leitura da tela de trabalho e da árvore.
O formulário servido é a fonte dos campos e do destino do POST.
O teste com transporte injetado confirma o orçamento e o efeito
incerto sem retry. Ainda não há comprovação live por instalação. O replay integral
de 15 cassettes foi executado em 05/11/2026, sem escrever no SEI.
A Web mantém a prova dos campos; POST fixo e REST não oferecem
a mesma garantia de origem dos dados.

## Proteção de redirecionamentos e prova de RED — PR #283

A confirmação 4.0 simulada requer **6 requisições**: três GET da
prévia, um POST criador e dois GET de verificação. Na variante 4.1/5.0
testada, a escolha de tipo adiciona um POST **não criador**: são
**7 requisições**, dois POST de efeitos distintos e dois GET finais.
Os dois fluxos rejeitam efeito incerto sem repetir a escrita.

O parser rejeita `Location` com duas ocorrências de `acao` (mesmo
quando uma é esperada), com duas ocorrências de `id_procedimento`
(inclusive uma inválida) ou sem ID numérico único. A lista de tipos
4.0 igualmente rejeita ambiguidade em `acao` e
`id_tipo_procedimento` antes de abrir o cadastro.

Os dois testes de redirecionamento foram publicados antes da correção
no commit `043ad9b`. No CI
[42094129597](https://github.com/franklinbaldo/ies/actions/runs/42094129597),
**2 testes falharam** por aceitarem o redirecionamento ambíguo;
1392 passaram e 13 constavam como ignorados **preexistentes**.
Essa é evidência de RED **no CI**, não execução no sandbox local.
Os testes adicionados não foram desabilitados.

## Evidências verificáveis

- [Contrato parcial](../../contracts/cli/processo-criar.md)
- [Matriz de evidência](../../contracts/evidencias/processo-criar.md)
- [Teste Rust da superfície CLI](../../tests/cli_processo_criar.rs)
- [Observação indireta](../como-o-sei-funciona/inicio-do-processo.md)

## Limitações e revisão

A CLI foi conectada ao fluxo Web, mas a validação ao vivo continua
NOT_RUN. O CI testa o fluxo com transporte sintético e nunca escreve
no SEI real. A hipótese legal AJAX já tem testes e replay offline;
a escolha e as diferenças entre instalações precisam de validação live. A escrita live foi autorizada na issue #245, exclusivamente em treinamento;
a prévia e a confirmação live continuam NOT_RUN enquanto não forem
executadas e documentadas. Reavaliar a escolha caso o replay mostre
cadastro não contemplado.

## Evidências e revisão

[Observação estrutural](../como-o-sei-funciona/inicio-do-processo.md);
[matriz do Todos](../../contracts/evidencias/processo-criar.md);
[issue #245](https://github.com/franklinbaldo/ies/issues/245).
Rever quando houver divergência no cadastro servido pela UFRJ/UnB
ou um replay que refute o redirecionamento observado pelo Todos.


## Fase AJAX da hipótese legal restrita — PR #283

Uma captura 4.0.9 do Todos @`323eb01` demonstra o
`infraAjaxMontarSelect` do `selHipoteseLegal` e sua URL assinada.
A mesma versão do oráculo consulta diretamente
`hipotese_legal_select_nome_base_legal` com os quatro campos literais,
sem executar JavaScript e sem consultar sugestões do tipo.
A hipótese legal ainda não está presente no select inicial (`null`).
A implementação IES usa `derivar_consulta_ajax` já existente, no
mesmo host e sessão, e interpreta apenas as opções efetivas da resposta
XML. O leitor offline estrito original continua recusando ausência de
hipótese. O fluxo permite que ela fique temporariamente pendente **só**
para buscar a lista publicada, antes de qualquer POST criador.

Na primeira fatia AJAX, a CLI não oferecia escolha explícita: exigia
uma única opção. A fatia posterior acrescentou `--hipotese-legal ID#NOME`,
conferido literalmente no XML da sessão. Sem a flag, continua exigida
uma única opção habilitada. Zero ou várias opções não autorizam escolha
automática; não escolher a primeira nem converter para público. O AJAX é um POST de
**consulta**, distinto do POST de criação, com uma tentativa.
Falha HTTP conserva o status e a mensagem saneada na CLI.

Quatro testes de regressão foram publicados antes desse código.
O RED foi observado no CI
[42097640389](https://github.com/franklinbaldo/ies/actions/runs/42097640389):
**4 falhas novas** (1396 passando; 13 ignorados preexistentes);
o RED e a compilação **locais** ficaram NOT_RUN porque não há
`cargo`/`rustc` no sandbox e o DNS impediu instalá-los.
O GREEN sintético inicial está no
[CI 42098895457](https://github.com/franklinbaldo/ies/actions/runs/42098895457).
Orçamentos demonstrados: prévia restrita em 4.0 com select inicialmente
vazio = **4 pedidos** (3 GET, um POST AJAX); confirmação restrita =
**7 pedidos** (5 GET, POST AJAX e POST criador), terminando em duas
leituras. HTTP 503 do AJAX e seleção ambígua não iniciam escrita.
Não extrapolar tais contagens ao SEI vivo, nem confundir esses testes
com replay do acervo.

O replay de 15 cassettes originais passou no ensaio 42102035089.
Seu teste, apesar do `#[ignore]` para ausência de ZIPs no teste Rust
sem rede, passou a ser executado obrigatoriamente no job
`replay-processo-criar` do CI, com aquisição e hash dos sete ZIPs.
A prévia/escrita/releitura ao vivo em treinamento continuam NOT_RUN. Não houve escrita em produção
nem em treinamento nesta fatia, logo não há resíduo.

## Seleção de hipótese legal múltipla por identidade observada

O replay público de 05/11/2026 encontrou respostas AJAX com mais
de uma hipótese legal na UnB. Escolher a primeira foi rejeitado:
a ordem não é prova de vontade ou fundamento jurídico. A alternativa
é um argumento explícito `--hipotese-legal ID#NOME`, que somente
é aceito para nível restrito, depois de comparar **ambas** as partes
com a resposta XML da sessão. ID duplicado, divergência de descrição,
ausência e opção desabilitada produzem recusas tipadas antes da escrita.
A seleção inline sem declaração AJAX continua fora deste recorte,
pois não foi acrescentada prova do nome dessa opção no leitor.

O custo é uma flag adicional à CLI e a necessidade de o operador
observar previamente as opções; o ganho é eliminar escolha arbitrária.
Os testes exercitam o mesmo `derivar_consulta_ajax` e os mesmos
orçamentos dos fluxos já existentes; o POST de criação continua único,
seguido de leitura para verificar o NUP. A evidencia é indireta
(cassette e oráculo Todos); até validação ao vivo em **treinamento**,
não alegar compatibilidade de escrita entre instalações. Revisar
quando houver observação de escolha inline com múltiplas opções.

## Diagnóstico de opções oferecidas após recusa segura

O replay da UnB demonstrou várias hipóteses AJAX habilitadas.
A escolha explícita `ID#NOME` só é praticável se o operador puder
conhecer os pares observados, sem inspeção de URLs assinadas ou logs
privados. A recusa `hipotese_legal_ajax_nao_unica` passa a devolver
os pares habilitados, com IDs únicos e texto imprimível, como
`hipoteses_legais_oferecidas` no JSON e `ID#NOME` no terminal.
É um diagnóstico de leitura, não pré-autorização de escrita.
IDs repetidos ou caracteres de controle não produzem sugestões;
a operação continua recusada mesmo na invocação com `--confirmar`.
Não executar JS nem emitir a URL assinada ou o cookie.

Alternativas rejeitadas: selecionar a primeira hipótese; publicar
o XML inteiro; exigir ao operador copiar informações da sessão.
A escolha precisa ser reconferida contra o AJAX de uma nova prévia,
com o mesmo orçamento sintético de quatro requisições; uma confirmação
posterior continua exigindo POST único e releitura verificadora.

**RED real:** [CI 42109336445](https://github.com/franklinbaldo/ies/actions/runs/42109336445)
registrou **três novos testes falhando** antes da implementação
(1423 passando, 14 ignorados preexistentes). Rustfmt e Clippy
também sinalizaram ajustes nos testes RED, a corrigir no GREEN.
RED local NOT_RUN (sandbox sem Rust/DNS). A prova ainda é sintética
até uma prévia autenticada no SEI de treinamento.
