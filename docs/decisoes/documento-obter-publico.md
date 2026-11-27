---
type: ArchitectureDecision
title: "Proposta #865: leitura pública de documento por link publicado"
description: "Decisão pendente do proprietário; escopo mínimo GET anônimo e prova explícita, sem habilitação live."
---

# Leitura pública por link publicado — proposta, não adotada

Refs [#865](https://github.com/franklinbaldo/ies/issues/865) e [R298](https://github.com/franklinbaldo/cafezinho/issues/21#issuecomment-7228989169).
Base examinada: `b4162991a6692bdaffd8be1650968fa38bc135fa`. Esta proposta não altera
a política de acesso, não declara comando implementado e não habilita produção.

## Evidência e alternativa

- A issue registra GET 200 do formulário de publicações de Rondônia, sem captcha
  observado no formulário, e a forma de ação `publicacao_visualizar`. A resposta
  do documento, o vínculo do ID com o número SEI e o rodapé do HTML não foram
  observados nesta rodada. Não montar URLs por número SEI.
- Os cards [UFRJ](../../cassettes/ufrj/documento-listar-publicacoes-ufrj--766527cd2023.md)
  e [UnB](../../cassettes/unb/documento-publicacao-unb--5ea56c71de51.md)
  registram publicacao_listar em fluxo autenticado, não GET anônimo de
  publicacao_visualizar. Nenhum card de leitura anônima foi localizado na árvore
  de cassettes consultada; disponibilidade das sete instalações permanece unknown.
- Conferência por verificador e CRC: captcha observado na issue em Rondônia;
  recusar, não enviar POST nem resolver captcha. Pesquisa pública também exige
  captcha nessa evidência. Pesquisa por publicações exige POST ainda não analisado.
  REST/SOAP requer autorização própria. Nenhum desses caminhos é fallback.
- Estratégia proposta: somente GET do link completo de visualização de publicação,
  previamente publicado pelo órgão e fornecido pelo chamador; resposta direta.
  Mais estreita e verificável que pesquisa automática. Não é prova de que o módulo
  esteja habilitado nem de que qualquer HTML retornado seja o documento original.

## Contrato mínimo submetido à aprovação

Sintaxe proposta:
`ies --json documento publico obter --origem BASE --link LINK --verificador V --crc C --saida ARQ`.

Um documento, um link de visualização HTTPS, origem/base explícita, par esperado
obtido de fonte independente pelo chamador; não usar o próprio rodapé como
expectativa. `--saida` obrigatório na primeira fatia para preservar bytes sem
misturá-los com JSON. O link deve pertencer à mesma base autorizada e ao caminho
de publicações descrito em #865, com ação de visualização explícita e identidade
única; não aceitar credenciais, fragmento, links assinados de sessão, parâmetros
ambíguos, redirecionamento, URL de conferência ou descoberta por IDs. A origem
autorizada é definida pela política, nunca apenas por informar `--origem`.
Sem alteração da lista de instalações nesta proposta.

Efeito remoto: somente um GET, sem login, cookie, Authorization ou cache de
sessão; não propagar Set-Cookie, não seguir redirect, não tentar novamente.
Reutilizar a fronteira HTTPS comum se o comportamento anônimo for demonstrado;
não criar um segundo cliente por preferência. Respeitar orçamento e autorização.
Sem POST, CAPTCHA, varredura, fallback ou acesso live nesta fase.

Sucesso somente após verificar tipo HTML, limite de corpo existente e rodapé
identificado inequivocamente, confrontando o par V/CRC esperado. Sem regra de
extração comprovada em HTML original, a forma do rodapé é hipótese de bancada.
Os bytes da entidade HTTP são preservados (inclusive charset e CRLF), separados
da representação decodificada usada pelo parser; hash SHA-256 e tamanho dos
mesmos bytes gravados. Arquivo 0600 e idempotência pelo mecanismo existente:
repetir bytes iguais não altera a foto; divergência não sobrescreve.
Nenhuma foto final em caso de recusa ou resposta incompleta.

JSON proposto: instalação, rota=publicacoes, id_documento (identidade opaca
do link, não número SEI), numero_sei somente se observado sem ambiguidade,
verificador, crc, content_type, charset quando declarado, sha256, tamanho,
obtido_em, autenticidade=confere-rodape. Esta última etiqueta significa
concordância do rodapé com a expectativa e origem autorizada, não verificação
criptográfica de assinatura ou confirmação pelo módulo de conferência.
Ausência do par ou prova insuficiente nunca retorna sucesso.

Recusas tipadas propostas: instalação não autorizada; entrada/link inválido;
captcha exigido; não encontrado; limite de taxa (429, ou bloqueio WAF
identificado); resposta inesperada (inclusive 3xx e 403 indeterminado);
rodapé ausente/ambíguo ou divergente; erro de arquivo. Reusar tipos existentes
quando semanticamente equivalentes, não classificar erros pelo texto.
Distinguir captcha por estrutura observável, não palavra solta no documento.
Os nomes finais dos tipos e subcódigos pertencem ao contrato aceito.

## Critérios de verificação

| Caso offline | Oráculo |
| --- | --- |
| HTML sintético ISO-8859-1 com CRLF e par correto | bytes, tamanho e SHA-256 esperados fixados fora do parser; arquivo idêntico |
| HTTP de bancada | um GET, ausência de Cookie e Authorization, nenhuma sessão local lida |
| 200 com formulário captcha, 200 com login ou rodapé ausente/duplicado | erro tipado; zero foto final e zero POST |
| CRC ou verificador divergente, inclusive identidade lexical diferente | recusa; não corrigir expectativa com a resposta |
| 404, 429, 403 não classificado, resposta truncada ou MIME incompatível | recusa tipada, nenhum retry |
| 302 para mesma base ou outra, URL de conferência, base não autorizada | sem follow-up; URL inválida recusada antes de rede |
| Arquivo existente idêntico/diferente | idempotência/recusa sem sobrescrita |

Testar o binário real em bancada HTTPS isolada, com fixture sintética rotulada.
Não usar cassettes de publicação autenticada como prova de leitura pública.
RED/GREEN/replay original/live: NOT_RUN. Para tornar o contrato observado, é
necessária uma captura autorizada e redigida do GET de visualização bem-sucedido,
com corpo, charset, proveniência, identidade e hash; capturas negativas conforme
disponibilidade. Não criar coletor nem executar captura aqui.

## Decisão requerida

Aprovação expressa do proprietário para: recorte exclusivamente por link
publicado, par independente obrigatório, destino de arquivo obrigatório,
sem redirects, sem prova de autenticidade mais forte que confere-rodape.
A regra de acesso à produção permanece separada e inalterada; nenhum acesso
real antes de sua modificação explícita e da coleta autorizada. Após aceite,
formalizar CommandContract e erros tipados, executar RED/GREEN em bancada e
submeter aos gates pertinentes. A issue #865 permanece aberta.
