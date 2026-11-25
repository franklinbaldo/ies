---
type: ImplementationDecision
title: Incluir documento interno pelo formulário publicado e conferir diferença da árvore
operation: documento.criar
implementation: partial
status: adopted-for-scope
scope: contrato, reconciliação em produção e fluxo WIP de submissão Web com uma escrita
selected_strategy: web-cadastro-e-diferenca-da-arvore
alternatives: web-cadastro-e-diferenca-da-arvore,web-confiar-em-id-da-resposta,rest-wssei
evidence_level: oraculo-todos-captura-unb-e-indices-de-cassette-go
live_validation: not-run
---

# Decisão para `documento criar` (issue #246)

## Contexto

O formulário deve ser observado por instalação, sem supor permissões ou campos.

## Alternativas consideradas

O objetivo é criar um documento nativo vazio, não uma cópia nem um
anexo externo. O Todos usa o link da barra, a lista de tipos e o
cadastro; a captura UnB 4.0.9 mostra `txtDescricao`,
`rdoNivelAcesso` e `lblSerieTitulo`.

1. **Web com formulário servido e diferença de árvores** (escolhida):
   exige um POST, seguido de leitura da árvore e confirmação de um
   único id e número SEI novo, preservando os existentes.
2. **Confiar no `id_documento` do Location**: economiza a conferência,
   mas não prova a presença de um novo documento nem expõe o número SEI.
3. **REST/WSSEI**: ainda sem evidência de disponibilidade e em
   desacordo com a prioridade Web do projeto.

## Comparação

| Critério | Web + árvore | Só Location | REST |
| --- | --- | --- | --- |
| Comprova número SEI na árvore | sim | não | não observado |
| Exige POST de criação | sim | sim | não avaliado |
| Usa URL assinada servida | sim | sim | não avaliado |
| Permite recusar escrita não confirmada | sim | não | não observado |

## Decisão e limites

Conferir a unidade e a situação antes de escrever, sem deduzir
permissões pelo botão. O tipo é escolhido pelo nome oferecido, com
recusas distintas por ausência e ambiguidade. O formulário real deve
ser derivado pelo primitivo compartilhado com efeito explícito, nunca
por `POST` montado à mão. Não repetir escrita por falha de rede.

A reconciliação não é um teste de quantidade apenas: os ids anteriores
precisam persistir; não podem surgir dois ids novos; o novo deve
publicar número SEI e tipo solicitado. Não aceitar "algum documento
novo" como êxito. Qualquer inconclusividade depois de enviar é 3.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-criar.md).
- [Evidência existente do acervo](../../contracts/evidencias/documento-criar.md).
- [Teste CLI](../../tests/cli_documento_criar.rs) (confere argumentos e falha fechada sem sessão).
- Testes unitários de leitura e reconciliação: `src/documento_criar/tests.rs`.
- Testes de derivação de formulário: `src/documento_criar/fluxo.rs`.

## Limitações e revisão

[Contrato](../../contracts/cli/documento-criar.md);
[observação](../como-o-sei-funciona/inclusao-documento-interno.md).
O acervo GO foi reproduzido no CI obrigatório com ZIP e membros verificados (15 respostas-alvo: 3 lidas, 5 recusadas, 7 não aplicáveis; [run 42127145672](https://github.com/franklinbaldo/ies/actions/runs/42127145672)). Isso não comprova a criação ao vivo nem as demais instalações.

Os testes sintéticos integrados cobrem 5 GET sem POST na prévia e 6 GET + 1 POST na confirmação com releitura. Um 302 sem Location, mesmo diante de alteração simultânea na árvore, não demonstra a consequência do POST: [RED 42131591459](https://github.com/franklinbaldo/ies/actions/runs/42131591459) seguido de correção para `EfeitoIncerto`, sem segundo POST. A compilação e os gates dessa correção precisam ser conferidos no head final.

A decisão será revista após validação autenticada em treinamento: prévia → um `--confirmar` → leitura independente, com número SEI, requisições e resíduo. Escrita autorizada em 05/11 exclusivamente em treinamento, ainda **NOT_RUN**; nenhuma requisição real nesta sessão.

## Revisão — guarda estrutural antes da escrita (05/11/2026)

A verificação antiga da atribuição do campo oculto aceitava
qualquer ocorrência textual na página, incluindo em marcação inerte.
A decisão agora é conferir o elemento `script` servido, seu tipo,
e a unicidade da instrução literal observada, sem executar JavaScript.
Uma forma de script não reconhecida falha fechada antes da escrita.
O teste sintético integrado verifica quatro falsos positivos de HTML
e exige cinco GET sem POST de criação.

Os erros da seleção e da prévia passaram a compor causas tipadas
com `#[derive(Causa)]` e preservar nomes legados e HTTP, inclusive
recusa da Infra. Nenhum destes testes prova a escrita ao vivo:
ela continua pendente exclusivamente em treinamento.


Na revisão da guarda, `script[src]` e strings/template literals JavaScript
foram reconhecidos como evidência textual inerte. O leitor ignora o corpo
inline de script externo e só aceita instrução literal em contexto de
código do script inline servido; não interpreta nem executa JS.
Os testes negativos impedem POST criador com orçamento **5 GET/0 POST**.

## Revisão — Location após o POST de criação

Uma Location 302 não vazia, isoladamente, não comprova que o POST provocou
a mudança observada: uma resposta para host externo ou outro caminho pode
coincidir com criação concorrente na árvore. O leitor passou a exigir
que a URL absoluta HTTPS ou relativa aponte para o mesmo
`controlador.php` da submissão e contenha exatamente uma ação não vazia.
A validação é sintática, sem seguir o redirecionamento ou executar JS.
Se a resposta não satisfizer essa condição, a verificação por leitura
ainda ocorre, mas o efeito permanece **incerto (saída 1)**, sem retry.

Testes sintéticos em `src/documento_criar/tests_confirmacao.rs` afirmam
um POST e um GET de releitura tanto na recusa da Location externa quanto
nos destinos locais absoluto/relativo, sem requisitar instalações reais.

## Revisão — ramos inatingíveis da guarda (05/11/2026)

A revisão do Gavião no head `c3d0cee` mostrou que uma linha idêntica à
atribuição de cadastro dentro de `if (false) { … }` era contada como
instrução ativa, habilitando incorretamente o POST. A guarda agora
reconhece somente a sequência literal direta e completa de
`function submeter()` observada na captura SEI 4.0.9:
`desabilitarBotaoSalvar(true);`, atribuição da flag e submissão do
formulário, sem blocos intermediários. Funções não reconhecidas,
aninhamentos, comentários, scripts externos, literais e definições
duplicadas falham fechados. É **reconhecimento de forma**, não um
interpretador de JavaScript.

Os testes integrados em `tests_integracao.rs` incluem ramos
inalcançáveis e exigem **5 GET / 0 POST** na recusa. O novo teste foi
publicado no commit `58839c8`, antes da correção; no
[CI 42158719053](https://github.com/franklinbaldo/ies/actions/runs/42158719053)
houve **1 teste FAIL, 1.659 PASS, 18 ignorados**. Os controles positivos
incluem o corpo da função `submeter` lido do oráculo @323eb01.
Os gates e o live do head corrigido requerem verificação própria;
não inferir efeito real do teste sintético.


## Revisão — atribuição isolada não autoriza escrita (06/11/2026)

O parecer 🦅 no head `951adfd` encontrou que uma atribuição isolada
podia passar na guarda mesmo sem `function submeter()`, desabilitação
do botão ou chamada de `frmDocumentoCadastro.submit()`. O script mínimo
era apenas fixture sintética; não é evidência da submissão do SEI.

A regressão de transporte foi publicada **antes da correção** no commit
`c18a463`. O [RED 42164978361](https://github.com/franklinbaldo/ies/actions/runs/42164978361)
registrou **1 FAIL, 1.671 PASS, 18 ignorados**. Uma atribuição isolada
causava tentativa de requisição além dos cinco GET roteirizados.
Agora a guarda exige a sequência completa observada, sem executar JS,
e a fixture positiva a representa. Recusa antes do POST: **5 GET exatos,
zero POST criador**. O sucesso continua condicionado à conferência por
leitura após no máximo uma escrita; live continua NOT_RUN.

## Revisão — botão Salvar realmente oferecido (06/11/2026)

**Alternativa rejeitada:** aceitar `function submeter()` sem controle
acionável no cadastro. Essa função pode existir como código inerte;
o POST manual do IES não deve substituir sem prova a ação oferecida
pela interface Web. **Estratégia adotada:** exigir a ligação literal do
botão `btnSalvar` à chamada `confirmarDados()` nas formas observadas,
recusando botão ausente, desabilitado ou contraditório. O oráculo
`documento_gerar.html @323eb01` mostra dois botões iguais (linhas
810 e 1061), portanto a guarda permite duplicação idêntica e não escolhe
arbitrariamente um deles. Usar `html_atributos`, não JS executado.
A restrição pode recusar formas válidas de outras instalações; só
ampliar mediante nova observação + teste. Ela protege contra falso
positivo antes do POST e mantém a confirmação por releitura posterior.
**RED:** [42170278378](https://github.com/franklinbaldo/ies/actions/runs/42170278378),
1 teste novo falhou antes da correção, 1.683 aprovados, 18 ignorados.
Live de treinamento ainda NOT_RUN, sem resíduo.

**Revisão adversarial (06/11):** um `onclick` no HTML não torna o
controle acionável se o próprio botão ou um ancestral tiver `hidden`,
ou se estiver dentro de `fieldset disabled`. Uma guarda DOM conservadora
recusa estes casos antes do POST, sem interpretar CSS ou executar JS,
mas aceita `fieldset` habilitado. A localização usa os limites dos nós
e os atributos pela peça `html_atributos`. São cenários sintéticos,
não estruturas afirmadas como observadas em treinamento.
[RED 42274703937](https://github.com/franklinbaldo/ies/actions/runs/42274703937):
1 teste novo FAIL/1.775 PASS/23 ignored; Rustfmt também apontou a
formatação do teste, corrigida depois, sem afrouxar gates.

## Revisão — comprovar ligação entre controle e escrita (06/11/2026)

**Alternativa rejeitada:** verificar isoladamente botão
`onclick="confirmarDados()"` e `function submeter()`: uma página
pode publicar ambos sem caminho de execução entre eles. **Decisão:**
exigir a cadeia direta da captura SEI 4.0.9: botão → `confirmarDados()`
→ `if (OnSubmitForm())` → `submeter()` → flag e `submit()`.
Função ausente, chamada divergente, ramo alterado e definições
duplicadas falham fechadas antes do POST, sem interpretar/executar JS.
[RED 42176956473](https://github.com/franklinbaldo/ies/actions/runs/42176956473):
1 teste FAIL e 1.684 PASS (18 ignored); a requisição excedeu
o orçamento sintético de 5 GET. Live treinamento segue NOT_RUN.

## Revisão — regex lexical em script alheio (06/11/2026)

**Observação pré-RED:** [diagnóstico UnB 42209629903](https://github.com/franklinbaldo/ies/actions/runs/42209629903)
mostrou dois botões Salvar ativos e a cadeia de três funções dentro de
um script inline clássico com mais de 600 linhas. O diagnóstico não
estabeleceu a causa exata da recusa pré-POST. A hipótese de uma expressão
regular em código auxiliar é **sintética**, não uma afirmação sobre o HTML
servido pela UnB.

**Regressão:** um segundo script auxiliar com regexp literal `/\\{/`
não altera a cadeia de Salvar nem redefine as funções relevantes.
O antigo scanner contava a chave escapada como abertura de bloco e
recusava a operação. Publicados testes antes da mudança nos commits
`0d1c819` e `a9d9098`. O [RED 42212158041](https://github.com/franklinbaldo/ies/actions/runs/42212158041)
registrou **1 falha nova / 1.749 PASS / 23 ignorados** na suíte Rust;
o job final da execução foi cancelado pela concorrência após gerar a
evidência do teste.

**Decisão:** distinguir somente regexp literal iniciado na expressão
de atribuição, inclusive escapes e classes de caracteres, no reconhecedor
léxico da guarda. Não executar ou avaliar JavaScript, não inferir prova
a partir do texto da regex e não alterar a sequência estrita
`btnSalvar → confirmarDados → OnSubmitForm → submeter → POST`.
Expressões desconhecidas continuam falhando fechadas. A hipótese não
autoriza POST live sem observação segura, prévia nova e gates verdes.


## Revisão — script extenso da UnB e texto truncado (06/11/2026)

Observação **antes do RED**: no ensaio somente leitura em TREINAMENTO
[42222453651](https://github.com/franklinbaldo/ies/actions/runs/42222453651),
a prévia PASS usou quatro requisições, enquanto a confirmação instrumentada
recusou **antes de qualquer POST criador**. O HTML integral ficou apenas
em arquivo temporário privado 0600, sem artifact nem log. Dois botões Salvar
publicaram o acionador observado. A leitura independente do script bruto
por HTMLParser identificou confirmarDados e submeter na cadeia esperada;
porém tl::HTMLTag::inner_text, que alimentava a guarda Rust, devolveu
**só 15 bytes/2 linhas** para o script ativo de índice 24, e a contagem
de funções ficou 0/0/0. Isso demonstra uma divergência **de extração do
texto JavaScript**, não permissão para relaxar a verificação semântica.

**Implementação na fatia:** identificar primeiro o `script` ativo no DOM,
validando `type`, `src` e exclusão de `template`, e recuperar sua fonte
literal no HTML pela posição inicial validada. O `inner_text` do tl não é
representação adequada de código JS: a remoção de markup pode apagar
instruções dentro de strings. A extração exige abertura inequívoca,
aspas fechadas nos atributos e primeiro fechamento literal
`</script>` (regra HTML, inclusive se aparecer dentro de uma string JS).
Sem fechamento seguro, recusar antes do POST. A guarda lexical continua
exigindo a cadeia inteira, sem executar JS, regex nem inserir requisições.
O teste sintético de `54a7ac1` passou sem RED; **a correção real na UnB
permanece a verificar** em nova prévia de treinamento antes de decidir
sobre qualquer confirmação. O diagnóstico não produziu resíduo.

## Revisão — distinguir aceite da coincidência temporal (06/11/2026)

**Problema observado em revisão do head `76912e7`:** o HTML 200 era
tratado automaticamente como aceite de inclusão. Numa conta compartilhada,
um outro agente pode ter incluído documento do mesmo tipo entre as fotos,
enquanto o POST do IES apenas reapresentou o cadastro. A diferença da
árvore prova existência, **não causalidade**. Na mesma fronteira,
o erro do envio era descartado, inclusive status HTTP e mensagem saneada.

**Alternativa rejeitada:** concluir por qualquer HTML 200 ou 302 para
uma ação arbitrária do controlador. **Decisão:** aceitar como indício
positivo apenas 302 para `documento_visualizar` com `id_documento`
único e correspondente ao nó novo da árvore. HTML 200 sem marcador de
aceite observado é inconclusivo. Não executar scripts nem seguir o 302.
Se o POST falhar, ainda reler a árvore, mas devolver saída 1 com causa
tipada original do envio e diagnóstico HTTP, sem retry. Preservar o
subcódigo público `nao_confirmado_na_criacao_de_documento`.
A validação real de retorno da instalação continua pendente; não inventar
marcador HTML de sucesso.

**RED antes do miolo:** [run 42229169889](https://github.com/franklinbaldo/ies/actions/runs/42229169889),
**3 FAIL novos, 1.753 PASS, 23 ignorados**: HTML 200 com criação
concorrente, erro 403 e erro 503 descartados. Os testes são sintéticos,
com uma chamada POST e uma GET de releitura, sem repetição.

A prova de funções de navegação ativas para seleção de tipo e filtro
está documentada em [scripts de navegação](../como-o-sei-funciona/documento-criar-scripts-ativos.md).

## Revisão — ablação do pseudo-interpretador JS (07/11/2026)
A [decisão do 🦡 Texugo na PR #248](https://github.com/franklinbaldo/ies/pull/248#issuecomment-6701907481)
revogou a exigência de equivalência semântica com o navegador. A submissão
é derivada do formulário real pelo primitivo compartilhado e exige um
controle Salvar ativo e não contraditório na tela observada.
O reconhecedor de funções globais, regex, MIME e passos de submeter()
foi removido da guarda de escrita, sem executar JS nem inventar campos.

**Cobertura dispensada de propósito:** oito testes semânticos de scripts
em tests_integracao.rs e dois em tests_scripts_ativos.rs. O IES não promete
detectar redefinições de callbacks JS. Mantidos os testes de controles
ausentes/divergentes/ocultos, fieldset disabled, template inerte,
identidade do processo, oferta do tipo, erros tipados e formulário servido.
Permanece a verificação causal: um POST, 302 para documento_visualizar
com id_documento correspondente, releitura independente do número SEI
e saída 1 após escrita não confirmada, sem retry. O leitor de script usado para
navegação entre tipos continua em guarda_navegacao.rs nesta fatia.

**Risco conscientemente aceito:** a lógica do navegador pode divergir
do envio HTTP construído pela CLI, que o próprio servidor pode recusar.
Não se aceita falso sucesso, origem divergente ou perda de causalidade.
**RED comprovado:** [CI 42340303462](https://github.com/franklinbaldo/ies/actions/runs/42340303462),
um teste novo FAIL, 1.882 PASS e 24 ignored; o teste-only ultrapassou
temporariamente 20 KB, também flagrado pelo gate. A ablação elimina
essa ultrapassagem. Green e live devem ser medidos no novo head.

